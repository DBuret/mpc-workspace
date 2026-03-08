use serde::Deserialize;

/// Configuration chargée automatiquement depuis les variables d'environnement.
/// Serde et Envy vont chercher les clés en majuscules correspondantes (ex: MCP_TPL_URL).
#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
	
    /// Le port d'écoute (MCP_TPL_PORT)
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// Le niveau de log (MCP_TPL_LOG)
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    pub url: String,
    
}


fn default_port() -> u16 {
    3000 
}

fn default_log_level() -> String {
    "info".into()
}

fn default_url() -> String {
    "http://172.17.0.1:18080".into()
}
