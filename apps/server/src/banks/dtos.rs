use crate::{banks::QuestionBankQuestion, courses::CourseId};

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "validate_question", skip_on_field_errors = false))]
pub struct QuestionInput {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Prompt must be between 1 and 1000 characters"
    ))]
    pub prompt: String,

    #[validate(length(min = 2, max = 5, message = "There must be between 2 and 5 options"))]
    pub options: Vec<String>,
    pub answer_index: usize,

    #[validate(length(max = 5, message = "There can be at most 5 images per question"))]
    pub images: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateQuestionBankDto {
    pub course_id: CourseId,
    #[validate(length(
        min = 1,
        max = 120,
        message = "Name must be between 1 and 120 characters"
    ))]
    pub name: String,
    #[validate(length(
        min = 1,
        max = 500,
        message = "There must be at least 1 and at most 500 questions"
    ))]
    #[validate(nested)]
    pub questions: Vec<QuestionInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "validate_update_has_fields", skip_on_field_errors = false))]
pub struct UpdateQuestionBankDto {
    #[validate(length(
        min = 1,
        max = 120,
        message = "Name must be between 1 and 120 characters"
    ))]
    pub name: Option<String>,
    #[validate(length(
        min = 1,
        max = 500,
        message = "There must be at least 1 and at most 500 questions"
    ))]
    #[validate(nested)]
    pub questions: Option<Vec<QuestionInput>>,
}

fn validate_question(question: &QuestionInput) -> Result<(), ValidationError> {
    if question.answer_index < question.options.len() {
        return Ok(());
    }

    let mut err = ValidationError::new("invalid_answer_index");
    err.message = Some("answerIndex must be a valid option index".into());
    Err(err)
}

fn validate_update_has_fields(dto: &UpdateQuestionBankDto) -> Result<(), ValidationError> {
    if dto.name.is_some() || dto.questions.is_some() {
        return Ok(());
    }

    let mut err = ValidationError::new("empty_update");
    err.message = Some("At least one field must be provided".into());
    Err(err)
}

impl From<&QuestionInput> for QuestionBankQuestion {
    fn from(value: &QuestionInput) -> Self {
        QuestionBankQuestion::builder()
            .prompt(value.prompt.clone())
            .options(value.options.clone())
            .answer_index(value.answer_index as i16)
            .images(value.images.clone())
            .build()
    }
}
