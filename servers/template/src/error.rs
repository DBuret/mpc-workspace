use thiserror::Error;

/// TODO: Définissez ici les erreurs métiers propres à cet agent.
/// Elles seront converties en messages d'erreur formatés pour l'IA.
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("API request failed: {0}")]
    ApiError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}
