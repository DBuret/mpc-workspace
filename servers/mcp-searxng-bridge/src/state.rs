use crate::config::AgentConfig;
use tokio::sync::broadcast;

/// L'état métier de votre serveur. C'est ici que vous stockez vos pools 
/// de base de données, vos clients HTTP, ou vos caches.
#[derive(Clone)]
pub struct AppState {
    // TO DO: Ajoutez vos clients métier ici (ex: reqwest::Client, sqlx::PgPool)
    pub client: reqwest::Client,
    
    // Requis par le noyau réseau pour le mode SSE
    pub tx: broadcast::Sender<String>,
}

impl AppState {
    /// Construit l'état à partir de la configuration environnementale
    pub fn new(config: &AgentConfig, tx: broadcast::Sender<String>) -> Self {
        
        // TO DO: Initialisez vos clients asynchrones ici
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("MCP-SearXNG-Bridge/1.0")
            .build()
            .expect("Failed to create reqwest client");
	    
	    let url = config.url;
        Self {
		url,
            client,
            tx,
        }
    }
}
