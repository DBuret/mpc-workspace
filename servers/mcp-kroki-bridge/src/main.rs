#![deny(unsafe_code)]

mod config;
mod error;
mod handlers;
mod state;

use mcp_network_core::{McpServer, create_mcp_router};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info,warn, error};


use crate::config::AgentConfig;
//use crate::error::AgentError;
use crate::handlers::{generate_url};
use crate::state::AppState;

/// Agent générique qui implémente le protocole MCP
#[derive(Clone)]
pub struct Agent {
    state: Arc<AppState>,
}

#[axum::async_trait]
impl McpServer for Agent {
    /// 📋 Auto-généré depuis Cargo.toml
    fn server_info(&self) -> Value {
        json!({ 
            "name": env!("CARGO_PKG_NAME"), 
            "version": env!("CARGO_PKG_VERSION")
        })
    }

    /// TODO: this mcp server tools description
    async fn list_tools(&self) -> Value {
        json!({
             "tools": [
            {
                "name": "render_plantuml",
                "description": "Génère des schémas d'architecture, séquences et classes via PlantUML.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string", "description": "Code PlantUML (ex: @startuml...)" }
                    },
                    "required": ["source"]
                }
            },
            {
                "name": "render_vega",
                "description": "Generates data charts (bar, line, pie, etc.) using a Vega-Lite JSON specification. IMPORTANT: the provided JSON must be strictly valid. All numeric values must be pre-computed literals (e.g. 10521.96). Arithmetic expressions like '4332.57 + 6189.39' are FORBIDDEN and will cause an error. Aggregate and compute all values BEFORE building the JSON spec. The JSON must have a single root object. Properties like name inside data must be inside the data object, not after its closing brace. Double-check all braces are correctly nested before submitting.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": {
                            "type": "string",
                            "description": "A strictly valid Vega-Lite JSON specification. All numeric values must be literals (number type), never expressions. The JSON must be directly parseable as-is."
                        }
                    },
                    "required": ["source"]
                }
            }
            ]
        })
    }

    /// TODO: this mcp server tools routing
    async fn call_tool(&self, name: &str, args: Option<&Value>) -> Result<String, String> {
        
        let source = args
            .and_then(|a| a.get("source"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        info!("tools/call: tool={}, source_len={}", tool_name, source.len());

        let kroki_type = match tool_name {
            "render_plantuml" => "plantuml",
            "render_vega" => "vegalite",
            _ => return Err(format!("Unknown tool '{}'", name))
        };

        // Vega-Lite validation
        if kroki_type == "vegalite" {
            if let Err(e) = serde_json::from_str::<serde_json::Value>(source) {      
                warn!("render_vega error: {}", e);
                return Err(format!("Invalid Vega-Lite JSON: {}", e))
            }
        }

        // Generate Kroki URL
        let url = generate_url(&state.kroki_url, kroki_type, source);
        let result = json!({
            "content": [{
                "type": "text",
                "text": format!("diagram url: {}", url)
            }]
        });

	return Json(McpResponse { jsonrpc: "2.0".into(), id: request_id, result }).into_response()
    }
    
}

/// Point d'entrée principal
#[tokio::main]
async fn main() {
    // 1. 🚀 Configuration auto-magique

    let env_prefix = format!(
        "{}_",
        env!("CARGO_PKG_NAME").to_uppercase().replace('-', "_")
    );

    let config: AgentConfig = match envy::prefixed(&env_prefix).from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Configuration failed: {}", e);
            eprintln!("📋 Expected env vars to be prefixed with '{}'", env_prefix);
            std::process::exit(1);
        }
    };

    // 2. 📊 Logs configurés
    tracing_subscriber::fmt()
        .with_env_filter(&config.log)
        .init();

    info!(config = ?config, "🚀 Configuration chargée avec succès");

    // 3. 🔄 État partagé
    let (tx, _) = broadcast::channel(100);
    let state = Arc::new(AppState::new(&config, tx.clone()));

    // 4. 🌐 Routeur MCP
    let agent = Agent { state };
    let app = create_mcp_router(agent, tx);

    // 5. 🚀 Démarrage
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    info!(
        "🚀 {} v{} started on {}",
        env_prefix,
        env!("CARGO_PKG_VERSION"),
        addr
    );

match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            if let Err(e) = axum::serve(listener, app).await {
                error!("❌ Erreur fatale du serveur: {}", e);
            }
        }
        Err(e) => {
            error!("❌ Impossible de lier le port {}: {}", config.port, e);
        }
    }
}
