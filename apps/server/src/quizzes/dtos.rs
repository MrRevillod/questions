use crate::quizzes::*;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "validate_create_schema", skip_on_field_errors = false))]
pub struct CreateQuizRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Title must be between 1 and 100 characters"
    ))]
    pub title: String,

    pub mode: String,
    pub start_time_utc: String,
    pub collaborator_ids: Vec<Uuid>,

    #[validate(range(
        min = 1,
        max = 240,
        message = "Duration must be between 1 and 240 minutes"
    ))]
    pub attempt_duration_minutes: i32,

    #[validate(range(
        min = 1,
        max = 100,
        message = "Question count must be between 1 and 100"
    ))]
    pub question_count: i32,

    #[validate(length(
        min = 1,
        max = 100,
        message = "There must be at least 1 question and at most 100 questions"
    ))]
    #[validate(nested)]
    pub questions: Vec<CreateQuizQuestionRequest>,

    #[validate(nested)]
    pub certainty_config: Option<CertainlyTableRequest>,
}

#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "validate_update_schema", skip_on_field_errors = false))]
pub struct UpdateQuizRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Title must be between 1 and 100 characters"
    ))]
    pub title: Option<String>,
    pub start_time_utc: Option<String>,

    #[validate(range(
        min = 1,
        max = 240,
        message = "Duration must be between 1 and 240 minutes"
    ))]
    pub attempt_duration_minutes: Option<i32>,

    #[validate(range(
        min = 1,
        max = 100,
        message = "Question count must be between 1 and 100"
    ))]
    pub question_count: Option<i32>,

    #[validate(length(
        min = 1,
        max = 100,
        message = "There must be at least 1 question and at most 100 questions"
    ))]
    #[validate(nested)]
    pub questions: Option<Vec<UpdateQuizQuestionRequest>>,

    #[validate(nested)]
    pub certainty_config: Option<CertainlyTableRequest>,
}

#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
pub struct CertainlyTableRequest {
    #[validate(nested)]
    pub low: CertainlyLevelRequest,
    #[validate(nested)]
    pub medium: CertainlyLevelRequest,
    #[validate(nested)]
    pub high: CertainlyLevelRequest,
}

#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
pub struct CertainlyLevelRequest {
    #[validate(range(
        min = 0,
        max = 100,
        message = "Correct values must be between 0 and 100"
    ))]
    pub correct: i16,
    #[validate(range(
        min = -100,
        max = 0,
        message = "Incorrect values must be between -100 and 0"
    ))]
    pub incorrect: i16,
}

#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[validate(schema(function = "validate_answer_index", skip_on_field_errors = false))]
pub struct CreateQuizQuestionRequest {
    pub answer: usize,

    #[validate(length(
        min = 1,
        max = 1000,
        message = "Question must be between 1 and 1000 characters"
    ))]
    pub question: String,

    #[validate(length(min = 2, max = 5, message = "There must be between 2 and 5 options"))]
    pub options: Vec<String>,

    #[validate(length(max = 5, message = "There can be at most 5 images per question"))]
    pub images: Vec<String>,
}

#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[validate(schema(
    function = "validate_question_id_and_answer",
    skip_on_field_errors = false
))]
pub struct UpdateQuizQuestionRequest {
    pub question_id: Option<Uuid>,
    pub answer: usize,
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Question must be between 1 and 1000 characters"
    ))]
    pub question: String,
    #[validate(length(min = 2, max = 5, message = "There must be between 2 and 5 options"))]
    pub options: Vec<String>,
    #[validate(length(max = 5, message = "There can be at most 5 images per question"))]
    pub images: Vec<String>,
}

#[derive(Clone, Debug, Validate, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinQuizByCodeRequest {
    #[validate(length(min = 3, max = 32, message = "Code length is invalid"))]
    pub code: String,
}

#[derive(Clone, Debug, Validate, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCollaboratorRequest {
    pub user_id: Uuid,
}

fn validate_create_schema(request: &CreateQuizRequest) -> Result<(), ValidationError> {
    validate_start_time(&request.start_time_utc)?;
    validate_quiz_mode(&request.mode, request.certainty_config.is_some())?;
    validate_question_count(request.question_count, request.questions.len())
}

fn validate_update_schema(request: &UpdateQuizRequest) -> Result<(), ValidationError> {
    if let Some(time) = &request.start_time_utc {
        validate_start_time(time)?
    }

    if let (Some(question_count), Some(questions)) =
        (request.question_count, request.questions.as_ref())
    {
        validate_question_count(question_count, questions.len())?;
    }

    Ok(())
}

fn validate_question_count(question_count: i32, bank_size: usize) -> Result<(), ValidationError> {
    if question_count <= 0 {
        return Ok(());
    }

    if question_count as usize <= bank_size {
        return Ok(());
    }

    let mut err = ValidationError::new("invalid_question_count");
    err.message = Some("questionCount cannot be greater than total questions in the bank".into());
    Err(err)
}

fn validate_start_time(start_time_utc: &str) -> Result<(), ValidationError> {
    if let Ok(start_time) = DateTime::parse_from_rfc3339(start_time_utc) {
        if start_time < Utc::now() {
            let mut err = ValidationError::new("start_time_in_past");
            err.message = Some("startTimeUtc cannot be in the past".into());
            return Err(err);
        }

        return Ok(());
    }

    let mut err = ValidationError::new("invalid_start_time_format");
    err.message = Some("startTimeUtc must be a valid RFC3339 datetime string".into());

    Err(err)
}

fn validate_quiz_mode(mode: &str, has_certainty_config: bool) -> Result<(), ValidationError> {
    match (mode, has_certainty_config) {
        ("certainty", true) => Ok(()),
        ("certainty", false) => {
            let mut err = ValidationError::new("certainty_config_required");
            err.message = Some("certaintyConfig is required for certainty quizzes".into());
            Err(err)
        }
        ("traditional", true) => {
            let mut err = ValidationError::new("certainty_config_not_allowed");
            err.message = Some("certaintyConfig is not allowed for traditional quizzes".into());
            Err(err)
        }
        ("traditional", false) => Ok(()),
        _ => {
            let mut err = ValidationError::new("invalid_mode");
            err.message = Some("mode must be 'traditional' or 'certainty'".into());
            Err(err)
        }
    }
}

fn validate_answer_index(question: &CreateQuizQuestionRequest) -> Result<(), ValidationError> {
    if question.answer < question.options.len() {
        return Ok(());
    }

    let mut err = ValidationError::new("invalid_answer_index");
    err.message = Some("Answer must be a valid index of options".into());

    Err(err)
}

fn validate_question_id_and_answer(q: &UpdateQuizQuestionRequest) -> Result<(), ValidationError> {
    if q.answer >= q.options.len() {
        let mut err = ValidationError::new("invalid_answer_index");
        err.message = Some("Answer must be a valid index of options".into());

        return Err(err);
    }

    Ok(())
}

impl From<CertainlyTableRequest> for CertainlyTable {
    fn from(value: CertainlyTableRequest) -> Self {
        Self {
            low: value.low.into(),
            medium: value.medium.into(),
            high: value.high.into(),
        }
    }
}

impl From<CertainlyLevelRequest> for CertainlyLevel {
    fn from(value: CertainlyLevelRequest) -> Self {
        Self {
            correct: value.correct,
            incorrect: value.incorrect,
        }
    }
}

impl FromStr for QuizKind {
    type Err = QuizError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "traditional" => Ok(QuizKind::Traditional),
            "certainty" => Ok(QuizKind::Certainly),
            _ => Err(QuizError::InvalidQuizMode),
        }
    }
}

impl From<&CreateQuizQuestionRequest> for QuizQuestion {
    fn from(value: &CreateQuizQuestionRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            question: value.question.clone(),
            options: value.options.clone(),
            answer: value.answer as i16,
            images: value.images.clone(),
        }
    }
}

impl From<&UpdateQuizQuestionRequest> for QuizQuestion {
    fn from(value: &UpdateQuizQuestionRequest) -> Self {
        Self {
            id: value.question_id.unwrap_or(Uuid::new_v4()),
            question: value.question.clone(),
            options: value.options.clone(),
            answer: value.answer as i16,
            images: value.images.clone(),
        }
    }
}
