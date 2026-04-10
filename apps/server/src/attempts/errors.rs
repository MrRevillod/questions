use sword::web::HttpError;
use thiserror::Error;

#[derive(Debug, Error, HttpError)]
pub enum AttemptError {
    #[http(code = 404, message = "The requested attempt was not found.")]
    #[error("Attempt not found: {0}")]
    NotFound(String),

    #[http(
        code = 404,
        message = "No in-progress attempt was found for this quiz."
    )]
    #[error("Active attempt not found for quiz: {0}")]
    ActiveAttemptNotFound(String),

    #[http(code = 400, message = "The provided attempt ID is invalid.")]
    #[error("Invalid attempt ID")]
    InvalidAttemptId,

    #[http(code = 400, message = "The provided question ID is invalid.")]
    #[error("Invalid question ID")]
    InvalidQuestionId,

    #[http(code = 403, message = "You do not have access to this attempt.")]
    #[error("Forbidden attempt access")]
    Forbidden,

    #[http(code = 409, message = "The attempt has already been submitted.")]
    #[error("Attempt already submitted")]
    AlreadySubmitted,

    #[http(code = 409, message = "The attempt is still in progress.")]
    #[error("Attempt has not been submitted yet")]
    NotSubmitted,

    #[http(code = 409, message = "The quiz attempt has not started yet.")]
    #[error("Quiz attempt cannot start before quiz start time")]
    NotStarted,

    #[http(code = 409, message = "The attempt has expired.")]
    #[error("Attempt expired")]
    Expired,

    #[http(code = 409, message = "The question does not belong to this attempt.")]
    #[error("Question does not belong to attempt")]
    QuestionNotInAttempt,

    #[http(
        code = 400,
        message = "The answer index is invalid for the selected question."
    )]
    #[error("Invalid answer index for question")]
    InvalidAnswerIndex,

    #[http(code = 404, message = "Attempt result is not available yet.")]
    #[error("Attempt result not available")]
    ResultNotAvailable,

    #[http(code = 409, message = "This attempt result has already been viewed.")]
    #[error("Attempt result already viewed")]
    ResultAlreadyViewed,
}
