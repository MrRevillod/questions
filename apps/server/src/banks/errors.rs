use sword::web::HttpError;
use thiserror::Error;

#[derive(Debug, Error, HttpError)]
pub enum QuestionBankError {
    #[http(code = 404, message = "The requested question bank was not found.")]
    #[error("Question bank not found: {0}")]
    NotFound(String),

    #[http(code = 400, message = "The provided question bank ID is invalid.")]
    #[error("Invalid question bank ID")]
    InvalidId,

    #[http(code = 403, message = "You do not have access to this question bank.")]
    #[error("Forbidden question bank access")]
    Forbidden,

    #[http(
        code = 409,
        message = "This bank is used by a quiz currently running and cannot be modified."
    )]
    #[error("Question bank is locked by running quiz")]
    LockedByRunningQuiz,

    #[http(
        code = 409,
        message = "The resulting snapshot has fewer questions than the quiz questionCount."
    )]
    #[error("Invalid question count after bank update")]
    InvalidQuestionCountAfterBankUpdate,

    #[http(
        code = 409,
        message = "Question bank snapshot was not found for one linked quiz."
    )]
    #[error("Question bank snapshot not found")]
    SnapshotNotFound,
}
