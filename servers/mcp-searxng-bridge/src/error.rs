use thiserror::Error;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

/// TODO: Définissez ici les erreurs métiers propres à cet agent.
/// Elles seront converties en messages d'erreur formatés pour l'IA.
#[derive(Error, Debug)]
pub enum AgentError {
	#[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Backend API error: {0}")]
    Api(String),
}

impl IntoResponse for AgentError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
