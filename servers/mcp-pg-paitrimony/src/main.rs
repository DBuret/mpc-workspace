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
use crate::handlers::{
    at_risk_positions, describe_table, list_tables, portfolio_performance, sector_exposure,
    sql_read_query,
};
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
        json!({"tools": [
            {
                 "name": "sql_read_query",
                    "description": concat!(
                        "Query the pAItrimony financial database.\n",
                        "⚠️ RULES:\n",
                        "1. NO SEMICOLON (;) at the end of the query.\n",
                        "2. ALL data tables (quotes, holdings, signals, news, corporateevents, analystratings) use ONLY the `isin` column as foreign key.\n",
                        "3. To search for a company, use `ticker` in the `assets` table. NOTE: One `ticker` can return multiple `isin` (e.g. employee stock plans vs direct shares). Look at `name` to distinguish.\n",
                        "4. To get the LATEST PRICE of any asset, ALWAYS use the view: `SELECT * FROM view_latest_quotes WHERE ticker = 'AAPL'`.\n",
                        "5. For portfolio analysis, use `view_portfolio_summary`."
                    ),
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "sql": {
                                "type": "string",
                                "description": "A valid SELECT SQL query without a trailing semicolon."
                            }
                        },
                        "required": ["sql"]
                    }
            },
            {
                "name": "list_tables",
                "description": concat!(
                    "List all tables and views available in the financial database. ",
                    "Call this first if you are unsure which tables exist."
                ),
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            },
            {
                "name": "describe_table",
                "description": concat!(
                    "Get the column definitions of a specific table or view in the financial database. ",
                    "Use this before writing a sql_read_query if you need to know ",
                    "the exact column names and types. ",
                    "Key tables: quotes, holdings, signals, news, view_portfolio_summary."
                ),
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "table": {
                            "type": "string",
                            "description": concat!(
                                "Table or view name, e.g. 'quotes', 'holdings', ",
                                "'signals', 'news', 'view_portfolio_summary'"
                            )
                        }
                    },
                    "required": ["table"]
                }
            },
            {
                "name": "portfolio_performance",
                "description": concat!(
                    "Returns ONLY the CURRENT PRESENT state of the user's investment portfolio: ",
                    "all positions across all accounts (PEA, CTO, Crypto, SCPI) ",
                    "with current market value, cost basis, unrealized profit/loss in currency and percentage. ",
                    "Uses view_portfolio_summary internally (isin-based joins). ",
                    "Use this to answer questions like: ",
                    "'How is my portfolio doing?', ",
                    "'What are my best/worst performing positions?', ",
                    "'What is my total portfolio value?'"
                ),
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            },
            {
                "name": "at_risk_positions",
                "description": concat!(
                    "Returns positions that are currently at risk, defined as: ",
                    "a loss exceeding the drawdown threshold (default: -10%) ",
                    "OR a negative average news sentiment over the last 7 days (below -0.5). ",
                    "Also returns RSI and moving averages (SMA50, SMA200) for each flagged position. ",
                    "Note: signals are joined via ticker, news via ticker. ",
                    "Use this to answer questions like: ",
                    "'What positions should I be worried about?', ",
                    "'Are there any alerts in my portfolio?', ",
                    "'Which stocks have bad news sentiment?'"
                ),
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "drawdown_threshold": {
                            "type": "number",
                            "description": "Loss percentage threshold (default: 10.0 — flags positions down more than 10%)"
                        }
                    },
                    "required": []
                }
            },
            {
                "name": "sector_exposure",
                "description": concat!(
                    "Returns the user's portfolio allocation broken down by market sector ",
                    "(Technology, Finance, Energy, etc.): number of positions per sector, ",
                    "total invested value, and percentage of the total portfolio. ",
                    "Joins via isin: holdings → assets. ",
                    "Use this to answer questions like: ",
                    "'Am I too exposed to Tech?', ",
                    "'What is my sector diversification?', ",
                    "'Should I rebalance my portfolio?'"
                ),
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            }
        ]})
    }

    /// TODO: this mcp server tools routing
   /// TODO: this mcp server tools routing
    async fn call_tool(&self, name: &str, args: Option<&Value>) -> Result<String, String> {
        match name {
            "sql_read_query" => {
                let sql = args.and_then(|a| a.get("sql")?.as_str()).unwrap_or("");
                sql_read_query(&self.state, sql).await.map_err(|e| e.to_string())
            }
            "list_tables" => {
                list_tables(&self.state).await.map_err(|e| e.to_string())
            }
            "describe_table" => {
                let table = args.and_then(|a| a.get("table")?.as_str()).unwrap_or("");
                describe_table(&self.state, table).await.map_err(|e| e.to_string())
            }
            "portfolio_performance" => {
                portfolio_performance(&self.state).await.map_err(|e| e.to_string())
            }
            "sector_exposure" => {
                sector_exposure(&self.state).await.map_err(|e| e.to_string())
            }
            "at_risk_positions" => {
                let threshold = args
                    .and_then(|a| a.get("drawdown_threshold")?.as_f64())
                    .unwrap_or(10.0);
                at_risk_positions(&self.state, threshold).await.map_err(|e| e.to_string())
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
    let state = Arc::new(AppState::new(&config, tx.clone()).await);

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
