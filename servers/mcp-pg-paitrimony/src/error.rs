use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// TODO: Définissez ici les erreurs métiers propres à cet agent.
/// Elles seront converties en messages d'erreur formatés pour l'IA.
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

        #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AgentError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
