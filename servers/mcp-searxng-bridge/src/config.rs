use serde::Deserialize;

/// Configuration chargée automatiquement depuis les variables d'environnement.
/// Serde et Envy vont chercher les clés en majuscules correspondantes

#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
    /// Le port d'écoute
    pub port: u16,

    /// Le niveau de log
    pub log: String,

    // searxng url
    pub url: String,
}
