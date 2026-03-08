use serde::Deserialize;

/// Configuration chargée automatiquement depuis les variables d'environnement.
/// Serde et Envy vont chercher les clés en majuscules correspondantes 

#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
	
    /// Le port d'écoute
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// Le niveau de log 
    #[serde(default = "default_log")]
    pub log: String,
    
    // searxng url
    #[serde(default = "default_url")]
    pub url: String,
    
}


fn default_port() -> u16 {
    3000 
}

fn default_log() -> String {
    "info".into()
}

fn default_url() -> String {
    "http://172.17.0.1:18080".into()
}
