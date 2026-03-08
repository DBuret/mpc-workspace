#![warn(missing_docs)]
#![deny(unsafe_code)]

mod config;
mod error;
mod handlers;
mod state;

use mcp_network_core::{create_mcp_router, McpServer};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::handlers::execute_tool_example;
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
                    "name": "example_tool",
                    "description": "Outil de template à remplacer",
                    "inputSchema": {
                        "type": "object",
                        "properties": { 
                            "input": { "type": "string", "description": "Paramètre d'entrée" }
                        },
                        "required": ["input"]
                    }
                }
                // TO DO: Ajoutez vos outils réels ici
            ]
        })
    }

    /// TODO: this mcp server tools routing
    async fn call_tool(&self, name: &str, args: Option<&Value>) -> Result<String, String> {
        match name {
            "example_tool" => {
                let input = args
                    .and_then(|a| a.get("input")?.as_str())
                    .unwrap_or("default");
                
                execute_tool_example(&self.state, input)
                    .await
                    .map_err(|e| e.to_string())
            }
            // TO DO: Ajoutez vos routes d'outils ici
            _ => Err(format!("Unknown tool '{}'", name)),
        }
    }
}

/// Point d'entrée principal
#[tokio::main]
async fn main() {
    // 1. 🚀 Configuration auto-magique
    let config: AgentConfig = match envy::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Configuration failed: {}", e);
            eprintln!("📋 Expected env vars prefixed with '{}'", 
                      env!("CARGO_PKG_NAME").to_uppercase().replace('-', "_"));
            std::process::exit(1);
        }
    };

    // 2. 📊 Logs configurés
    tracing_subscriber::fmt().with_env_filter(&config.log_level).init();

    // 3. 🔄 État partagé
    let (tx, _) = broadcast::channel(100);
    let state = Arc::new(AppState::new(&config, tx.clone()));
    
    // 4. 🌐 Routeur MCP
    let agent = Agent { state };
    let app = create_mcp_router(agent, tx);

    // 5. 🚀 Démarrage
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("🚀 {} v{} started on {}", 
          env!("CARGO_PKG_NAME"), 
          env!("CARGO_PKG_VERSION"), 
          addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
