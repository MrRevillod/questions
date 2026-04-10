use sword::web::HttpError;
use thiserror::Error;

#[derive(Debug, Error, HttpError)]
pub enum QuizError {
    #[http(code = 404, message = "The requested quiz was not found.")]
    #[error("Quiz not found: {0}")]
    NotFound(String),

    #[http(code = 400, message = "The provided quiz ID is invalid.")]
    #[error("Invalid quiz ID")]
    InvalidId,

    #[http(code = 400, message = "The provided quiz join code is invalid.")]
    #[error("Invalid quiz join code")]
    InvalidCode,

    #[http(code = 403, message = "You do not have access to this quiz.")]
    #[error("Forbidden quiz access")]
    Forbidden,

    #[http(code = 403, message = "Only the quiz owner can manage collaborators.")]
    #[error("Only the owner can manage collaborators")]
    OnlyOwnerCanManageCollaborators,

    #[http(
        code = 409,
        message = "The collaborator is already registered for this quiz."
    )]
    #[error("Collaborator already exists")]
    CollaboratorAlreadyExists,

    #[http(code = 404, message = "The collaborator was not found for this quiz.")]
    #[error("Collaborator not found")]
    CollaboratorNotFound,

    #[http(
        code = 400,
        message = "Only assistant and func users can be quiz collaborators."
    )]
    #[error("Invalid collaborator role")]
    InvalidCollaboratorRole,

    #[http(
        code = 409,
        message = "This quiz has been closed and no longer accepts attempts."
    )]
    #[error("Quiz is closed")]
    Closed,
}
