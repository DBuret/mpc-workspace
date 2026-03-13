use crate::config::AgentConfig;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio::sync::broadcast;

/// L'état métier de votre serveur. C'est ici que vous stockez vos pools
/// de base de données, vos clients HTTP, ou vos caches.
pub struct AppState {
    // TO DO: Ajoutez vos clients métier ici (ex: reqwest::Client, sqlx::PgPool)
    // pub custom_setting: String,
    // pub client: reqwest::Client,
    // pub url: String,

    // Requis par le noyau réseau pour le mode SSE
    pub tx: broadcast::Sender<String>,
    pub math_state: Mutex<MathState>,
}

impl AppState {
    /// Construit l'état à partir de la configuration environnementale
    pub fn new(config: &AgentConfig, tx: broadcast::Sender<String>) -> Self {
        // TO DO: Initialisez vos clients asynchrones ici
        //
        // let client = ...
        // let url = ...

        Self {
            // custom_setting,
            // client,
            // url,
            tx,
            math_state: Mutex::new(MathState::new()),
        }
    }
}

use mathexpr::Executable;
pub type CompiledExpr = Executable;

pub struct MathState {
    pub cache: HashMap<String, CompiledExpr>,
}

impl MathState {
    pub fn new() -> Self {
        MathState {
            cache: HashMap::new(),
        }
    }
}
