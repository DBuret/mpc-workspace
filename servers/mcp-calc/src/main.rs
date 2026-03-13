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
use crate::handlers::compute;
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
                "tools" : [
                {
                    "name": "evaluate",
                "description": "FOR MATH ONLY. MANDATORY tool for ALL calculations (even 1+1). Use this to avoid LLM arithmetic errors.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "The mathematical expression to evaluate. Use standard notation: +, -, *, /, ^, %. Built-in functions: sin, cos, tan, asin, acos, atan, sqrt, exp, log, log2, abs, ceil, floor, round. Built-in constants: pi, e. Examples: '2+2', 'sqrt(x^2 + y^2)', '2*pi*r', 'log(1000)/log(10)'."
                        },
                        "vars": {
                            "type": "object",
                            "description": "Named variables used in the expression. Example: {\"x\": 3.0, \"y\": 4.0, \"r\": 6371.0}. Omit if the expression has no variables.",
                            "additionalProperties": { "type": "number" }
                        }
                    },
                    "required": ["expression"]
                }
            }
        ]})
    }

    /// TODO: this mcp server tools routing
    async fn call_tool(&self, name: &str, args: Option<&Value>) -> Result<String, String> {
        match name {
            "evaluate" => {
                // 1. On attend (await) l'obtention du lock asynchrone
                let mut math_state = self.state.math_state.lock().await;

                // 2. On appelle le handler
                let result_json = compute(args.cloned(), &mut math_state);

                // 3. On renvoie le résultat
                Ok(serde_json::to_string(&result_json).unwrap())
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
