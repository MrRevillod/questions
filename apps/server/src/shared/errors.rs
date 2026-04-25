use crate::{
    attempts::AttemptError, authz::AuthzError, banks::QuestionBankError, courses::CoursesError,
    quizzes::QuizError, users::UsersError,
};

use jsonwebtoken::errors::Error as JwtError;
use ldap3::LdapError;
use sqlx::Error as SqlxError;
use std::io::Error as IoError;
use sword::web::*;
use thiserror::Error;

pub type AppResult<T = JsonResponse> = Result<T, AppError>;

#[derive(Debug, Error, HttpError)]
pub enum AppError {
    #[http(code = 403)]
    #[tracing(error)]
    #[error("Unauthorized: {0}")]
    Unauthorized(#[from] JwtError),

    #[http(transparent)]
    #[error(transparent)]
    Quiz(#[from] QuizError),

    #[http(transparent)]
    #[error(transparent)]
    Attempt(#[from] AttemptError),

    #[http(transparent)]
    #[error(transparent)]
    Users(#[from] UsersError),

    #[http(transparent)]
    #[error(transparent)]
    Courses(#[from] CoursesError),

    #[http(transparent)]
    #[error(transparent)]
    QuestionBank(#[from] QuestionBankError),

    #[http(transparent)]
    #[error(transparent)]
    Authz(#[from] AuthzError),

    #[http(
        code = 401,
        message = "The requested token was not found. Try again or generate a new one."
    )]
    #[error("Unauthorized access")]
    TokenNotFound,

    #[http(
        code = 401,
        message = "The provided token is invalid. Please try again or generate a new one."
    )]
    #[error("Invalid token")]
    InvalidToken,

    #[http(code = 500)]
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[http(code = 500)]
    #[error("IO error: {0}")]
    Io(#[from] IoError),

    #[http(code = 400)]
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[http(
        code = 401,
        message = "Invalid Credentials. Please try again, or contact support."
    )]
    #[error("LDAP Authentication failed: {0}")]
    LdapAuth(#[from] LdapError),

    #[http(
        code = 401,
        message = "No email address is associated with your account. Please contact support."
    )]
    #[error("LDAP Email not found")]
    LdapEmailNotFound,

    #[http(
        code = 401,
        message = "No username is associated with your account. Please contact support."
    )]
    #[error("LDAP Error: {0}")]
    LdapUsernameNotFound(String),
}
