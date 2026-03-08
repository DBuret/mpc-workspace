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
                    call_searxng(&self.state, input).await
                }
                "fetch_page" => {
                    let url = args.and_then(|a| a.get("url")?.as_str()).unwrap_or("");
                    fetch_url(&state, url).await
                }
            _ => Err(crate::error::AgentError::Api(format!("Unknown tool '{}'", name)),
        }
	//match res {
         //       Ok(t) => json!({ "content": [{ "type": "text", "text": t }] }),
         //       Err(e) => json_error(&e.to_string()),
        //    }
	    
	    //
    }
}

/// Point d'entrée principal
#[tokio::main]
async fn main() {
    // 1. 🚀 Configuration auto-magique
    
    let env_prefix = format!("{}_", env!("CARGO_PKG_NAME").to_uppercase().replace('-', "_"));
    
    let config: AgentConfig = match envy::prefixed(&env_prefix).from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Configuration failed: {}", e);
            eprintln!("📋 Expected env vars to be prefixed with '{}'", env_prefix);
            std::process::exit(1);
        }
    };
    

    // 2. 📊 Logs configurés
    tracing_subscriber::fmt().with_env_filter(&config.log).init();

    // 3. 🔄 État partagé
    let (tx, _) = broadcast::channel(100);
    let state = Arc::new(AppState::new(&config, tx.clone()));
    
    // 4. 🌐 Routeur MCP
    let agent = Agent { state };
    let app = create_mcp_router(agent, tx);

    // 5. 🚀 Démarrage
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("🚀 {} v{} started on {}", 
          env_prefix, 
          env!("CARGO_PKG_VERSION"), 
          addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
