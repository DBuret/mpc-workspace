use crate::config::AgentConfig;
use tokio::sync::broadcast;

use std::env;
use sqlx::PgPool;
use tracing::info;

/// L'état métier de votre serveur. C'est ici que vous stockez vos pools
/// de base de données, vos clients HTTP, ou vos caches.
#[derive(Clone)]
pub struct AppState {
    // TO DO: Ajoutez vos clients métier ici (ex: reqwest::Client, sqlx::PgPool)
    // pub custom_setting: String,
    // pub client: reqwest::Client,
    // pub url: String,

    // Requis par le noyau réseau pour le mode SSE
    pub tx: broadcast::Sender<String>,
        pub pool: PgPool,
}

impl AppState {
    /// Construit l'état à partir de la configuration environnementale
    pub async fn new(config: &AgentConfig, tx: broadcast::Sender<String>) -> Self {
        // TO DO: Initialisez vos clients asynchrones ici
        //
        // let client = ...
        // let url = ...

                
        let database_url = config.database_url.clone();

        info!("Connecting to PostgreSQL...");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to PostgreSQL");
        info!("PostgreSQL connection pool established");
        Self {
            // custom_setting,
            // client,
            // url,
            tx, 
            pool,
        }
    }
}
