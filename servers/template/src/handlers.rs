use crate::state::AppState;
use crate::error::AgentError;
use std::sync::Arc;

// TODO: implement tools in this file


/// Exemple d'implémentation d'un outil métier
pub async fn execute_hello_tool(
    _state: &Arc<AppState>, 
    name: &str
) -> Result<String, AgentError> {
    // TO DO: Implémentez votre logique ici (appels HTTP, requêtes SQL, etc.)
    
    if name.is_empty() {
        return Err(AgentError::ValidationError("Le nom ne peut pas être vide".into()));
    }
    
    Ok(format!("Bonjour {} ! Voici l'outil template.", name))
}
