#![deny(unsafe_code)]

mod config;
mod error;
mod handlers;
mod state;

use mcp_network_core::{McpServer, create_mcp_router};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::config::AgentConfig;
//use crate::error::AgentError;
use crate::handlers::{call_searxng, fetch_url};
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

    async fn list_tools(&self) -> Value {
        json!({
          "tools": [
            {
                "name": "search",
                "description": "Search the web via SearXNG",
                "inputSchema": {
                    "type": "object",
                    "properties": { "query": { "type": "string" } },
                    "required": ["query"]
                }
            },
            {
                "name": "fetch_page",
                "description": "Get the content of a web page as Markdown",
                "inputSchema": {
                    "type": "object",
                    "properties": { "url": { "type": "string" } },
                    "required": ["url"]
                }
            }
        ]
        })
    }

    /// TODO: this mcp server tools routing
    async fn call_tool(&self, name: &str, args: Option<&Value>) -> Result<String, String> {
        match name {
            "search" => {
                let input = args.and_then(|a| a.get("query")?.as_str()).unwrap_or("");
                call_searxng(&self.state, input)
                    .await
                    .map_err(|e| e.to_string())
            }
            "fetch_page" => {
                let url = args.and_then(|a| a.get("url")?.as_str()).unwrap_or("");
                fetch_url(&self.state, url).await.map_err(|e| e.to_string())
            }
            _ => Err(format!("Unknown tool '{}'", name)),
        }
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
