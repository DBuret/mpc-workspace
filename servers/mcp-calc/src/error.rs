use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Compile error: {0}")]
    Compile(String),
    #[error("Evaluation error: {0}")]
    Eval(String),
}

impl IntoResponse for AgentError {
    fn into_response(self) -> Response {
        // On convertit l'erreur en un tuple (Code de statut, Message)
        // Axum sait transformer ce tuple en une réponse HTTP valide.
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
