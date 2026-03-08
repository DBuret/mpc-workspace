use serde::Deserialize;

/// Configuration chargée automatiquement depuis les variables d'environnement.
/// Serde et Envy vont chercher les clés en majuscules correspondantes (ex: MCP_TPL_URL).
#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
    
    /// do not edit: listening port , log level
    pub port: u16,
    pub log: String,
    pub url: String
}


