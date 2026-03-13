use serde::Deserialize;

/// Configuration chargée automatiquement depuis les variables d'environnement.
/// Serde et Envy vont chercher les clés en majuscules correspondantes (ex: MCP_TPL_URL).
#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
    /// do not edit: listening port , log level
    pub port: u16,
    pub log: String,
    // TO DO: edit your env vars

    // Exemple : MCP_TPL_API_KEY
    // pub mcp_tpl_api_key: String,

    // Une variable optionnelle (ex: MCP_TEMPLATE_BRIDGE_API_KEY)
    // Si la variable n'est pas définie, api_key vaudra `None`.
    // pub api_key: Option<String>,

    // Vous pouvez aussi avoir des types numériques optionnels
    // (ex: MCP_TEMPLATE_BRIDGE_TIMEOUT)
    // pub timeout_seconds: Option<u32>,
}
