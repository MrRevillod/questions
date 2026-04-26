use sword::web::HttpError;
use thiserror::Error;

#[derive(Debug, Error, HttpError)]
pub enum UsersError {
    #[http(code = 403, message = "Only professors can assign assistants.")]
    #[error("Only professors can assign assistants")]
    OnlyProfessorsCanAssign,

    #[http(code = 404, message = "The requested user was not found.")]
    #[error("User not found: {0}")]
    NotFound(String),

    #[http(
        code = 400,
        message = "Only students and assistants can be managed from this endpoint."
    )]
    #[error("Only students and assistants can be managed")]
    InvalidAssistantTargetRole,

    #[http(code = 400, message = "Invalid user role provided.")]
    #[error("Invalid user role provided")]
    InvalidUserRole,
}
