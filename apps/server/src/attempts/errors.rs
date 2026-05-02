use sword::web::HttpError;
use thiserror::Error;
use uuid::Uuid;

use crate::attempts::AttemptId;

#[derive(Error, Debug, HttpError)]
pub enum AttemptError {
    #[http(code = 403, message = "El quiz no ha comenzado aún")]
    #[error("Quiz has not started yet")]
    QuizNotStarted,

    #[http(code = 403, message = "El quiz ha finalizado")]
    #[error("Quiz has already ended")]
    QuizEnded,

    #[http(
        code = 403,
        message = "Ya tienes un intento registrado para este quiz. Solo se permite un intento por quiz."
    )]
    #[error("User has already attempted this quiz")]
    AlreadyAttempted,

    #[http(code = 404, message = "Intento no encontrado")]
    #[error("Attempt not found")]
    NotFound(AttemptId),

    #[http(code = 403, message = "No estás autorizado para realizar esta acción")]
    #[error("User is not authorized to perform this action")]
    Forbidden,

    #[http(code = 403, message = "El intento ha expirado")]
    #[error("Attempt has expired")]
    Expired,

    #[http(code = 403, message = "El intento ya ha sido enviado")]
    #[error("Attempt has already been submitted")]
    AlreadySubmitted,

    #[http(code = 400, message = "La pregunta no pertenece a este intento")]
    #[error("Question does not belong to attempt: {0}")]
    QuestionNotInAttempt(Uuid),

    #[http(code = 403, message = "Los resultados aun no estan disponibles")]
    #[error("Results are not available yet")]
    ResultsNotAvailable,

    #[http(
        code = 400,
        message = "El indice de respuesta no es valido para la pregunta"
    )]
    #[error("Invalid answer index")]
    InvalidAnswerIndex,

    #[http(
        code = 400,
        message = "El nivel de certeza es obligatorio para este quiz"
    )]
    #[error("Certainty level is required for certainty quizzes")]
    CertaintyLevelRequired,

    #[http(code = 400, message = "El nivel de certeza no aplica para este quiz")]
    #[error("Certainty level is not allowed for traditional quizzes")]
    CertaintyLevelNotAllowed,

    #[http(code = 404, message = "No existe intento para este quiz")]
    #[error("Attempt not found for quiz")]
    NotFoundForQuiz,
}
