use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

// ==========================================
// 1. Modèles de Données MCP (JSON-RPC)
// ==========================================

#[derive(Deserialize, Debug, Clone)]
pub struct McpRequest {
    pub jsonrpc: Option<String>,
    pub method: String,
    pub id: Option<Value>,
    pub params: Option<Value>,
}

#[derive(Serialize, Debug, Clone)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    pub result: Value,
}

// ==========================================
// 2. Trait que chaque Agent IA doit implémenter
// ==========================================

#[axum::async_trait]
pub trait McpServer: Send + Sync + 'static {
    /// Informations retournées lors de la phase d'initialisation
    fn server_info(&self) -> Value;
    
    /// Liste des outils disponibles (retourne la structure JSON complète)
    async fn list_tools(&self) -> Value;
    
    /// Exécute un outil en fonction de son nom et de ses arguments
    async fn call_tool(&self, name: &str, args: Option<&Value>) -> Result<String, String>;
}

// ==========================================
// 3. État Interne du Routeur partagé
// ==========================================

/// Structure d'état générique encapsulant le serveur métier et le canal SSE
pub struct CoreState<S> {
    pub server: S,
    pub tx: broadcast::Sender<String>,
}

// ==========================================
// 4. Constructeur du Routeur Axum
// ==========================================

/// Crée un routeur Axum configuré avec toutes les routes MCP standard.
/// Accepte n'importe quel type `S` implémentant `McpServer`.
pub fn create_mcp_router<S>(server: S, tx: broadcast::Sender<String>) -> Router
where
    S: McpServer + Clone,
{
    // On wrap le tout dans un Arc pour le State Axum
    let state = Arc::new(CoreState { server, tx });

    Router::new()
        .route("/health", get(|| async { "OK" }))
        // LMStudio exige le POST sur /sse pour l'initialisation (Mode Hybride)
        .route(
            "/sse",
            get(sse_handler::<S>).post(messages_handler::<S>),
        )
        // Transport SSE standard
        .route("/messages", post(messages_handler::<S>))
        // Transport HTTP direct (ex: Open WebUI)
        .route("/mcp", post(mcp_handler::<S>))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// ==========================================
// 5. Handlers Réseau : SSE et Hybrid Response
// ==========================================

/// Endpoint de connexion SSE
async fn sse_handler<S>(
    State(state): State<Arc<CoreState<S>>>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    info!("New SSE connection requested");
    let rx = state.tx.subscribe();
    
    let stream = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(msg) => Some((Ok(Event::default().data(msg)), rx)),
            Err(_) => None, // Fin du flux si le channel est fermé ou en retard
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::new())
}

/// Endpoint de réception des messages (SSE / Mode Hybride)
async fn messages_handler<S>(
    State(state): State<Arc<CoreState<S>>>,
    headers: HeaderMap,
    Json(payload): Json<McpRequest>,
) -> impl IntoResponse
where
    S: McpServer + Clone,
{
    let method = payload.method.clone();
    let request_id = payload.id.clone().unwrap_or(Value::Null);

    // --- STRATÉGIE HYBRIDE POUR LMSTUDIO ---
    // Si c'est une initialisation, on répond DIRECTEMENT en HTTP 200.
    // Cela évite d'attendre l'ouverture du tunnel SSE qui arrive souvent trop tard.
    if method == "initialize" {
        info!("Handling 'initialize' via direct HTTP response (Hybrid mode)");
        
        let result = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": state.server.server_info()
        });
        
        let response = McpResponse {
            jsonrpc: "2.0".into(),
            id: request_id,
            result,
        };
        
        return (StatusCode::OK, Json(response)).into_response();
    }

    let tx = state.tx.clone();
    let server = state.server.clone();

    // Pour les appels d'outils, on utilise le spawn asynchrone + envoi SSE
    tokio::spawn(async move {
        // Ignorer les notifications d'initialisation sans ID
        if request_id.is_null() && method != "notifications/initialized" {
            return;
        }

        let result = match method.as_str() {
            "tools/list" => server.list_tools().await,
            "tools/call" => {
                let name = payload
                    .params
                    .as_ref()
                    .and_then(|p| p.get("name")?.as_str())
                    .unwrap_or("");
                let args = payload.params.as_ref().and_then(|p| p.get("arguments"));

                match server.call_tool(name, args).await {
                    Ok(t) => json!({ "content": [{ "type": "text", "text": t }] }),
                    Err(e) => {
                        error!("Tool call failed: {}", e);
                        json_error(&e)
                    }
                }
            }
            "notifications/initialized" => return, // Notification muette
            _ => json_error(&format!("Method {} not supported", method)),
        };

        // Formatage de la réponse
        let response = McpResponse {
            jsonrpc: "2.0".into(),
            id: request_id,
            result,
        };

        if let Ok(json_msg) = serde_json::to_string(&response) {
            // Tentative d'envoi via SSE avec retry (si le client s'est connecté entre-temps)
            let mut delivered = false;
            for _ in 0..3 {
                if tx.send(json_msg.clone()).is_ok() {
                    delivered = true;
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            if !delivered {
                warn!("Could not deliver {} via SSE (no client connected)", method);
            }
        }
    });

    StatusCode::ACCEPTED.into_response()
}

// ==========================================
// 6. Handler Réseau : HTTP POST direct
// ==========================================

/// Endpoint MCP classique (Requête/Réponse synchrone, utilisé par Open WebUI)
async fn mcp_handler<S>(
    State(state): State<Arc<CoreState<S>>>,
    Json(payload): Json<McpRequest>,
) -> impl IntoResponse
where
    S: McpServer + Clone,
{
    let method = payload.method.as_str();
    let request_id = payload.id.clone().unwrap_or(Value::Null);

    // 1. Gestion de l'initialisation
    if method == "initialize" {
        let result = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": state.server.server_info()
        });
        
        return Json(McpResponse {
            jsonrpc: "2.0".into(),
            id: request_id,
            result,
        })
        .into_response();
    }

    // 2. Routage des méthodes
    let result = match method {
        "tools/list" => state.server.list_tools().await,
        
        "tools/call" => {
            let name = payload
                .params
                .as_ref()
                .and_then(|p| p.get("name")?.as_str())
                .unwrap_or("");
            let args = payload.params.as_ref().and_then(|p| p.get("arguments"));

            match state.server.call_tool(name, args).await {
                Ok(t) => json!({ "content": [{ "type": "text", "text": t }] }),
                Err(e) => json_error(&e),
            }
        }

        "notifications/initialized" => return StatusCode::OK.into_response(),

        _ => json_error(&format!("Method {} not supported", method)),
    };

    // 3. Réponse JSON-RPC standard
    Json(McpResponse {
        jsonrpc: "2.0".into(),
        id: request_id,
        result,
    })
    .into_response()
}

// ==========================================
// 7. Utilitaires
// ==========================================

/// Formate une erreur compatible avec le protocole MCP
fn json_error(msg: &str) -> Value {
    json!({
        "isError": true,
        "content": [{ "type": "text", "text": msg }]
    })
}
