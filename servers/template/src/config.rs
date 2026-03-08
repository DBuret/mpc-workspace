use serde::Deserialize;

/// Configuration chargée automatiquement depuis les variables d'environnement.
/// Serde et Envy vont chercher les clés en majuscules correspondantes (ex: MCP_TPL_URL).
#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
    // TO DO: Définissez vos variables d'environnement ici
    
    /// Exemple : MCP_TPL_API_KEY
    // pub mcp_tpl_api_key: String,

    /// Le port d'écoute (MCP_TPL_PORT)
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// Le niveau de log (MCP_TPL_LOG)
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

// TO DO: Fonctions pour les valeurs par défaut
fn default_port() -> u16 {
    3000 // Changez le port par défaut si vous lancez plusieurs agents en même temps
}

fn default_log_level() -> String {
    "info".into()
}
