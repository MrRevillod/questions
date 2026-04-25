use crate::{
    banks::{QuestionBank, QuestionBankQuestion},
    courses::CourseId,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionBankQuestionView {
    pub id: Uuid,
    pub prompt: String,
    pub options: Vec<String>,
    pub answer_index: i16,
    pub images: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionBankView {
    pub id: String,
    pub course_id: CourseId,
    pub name: String,
    pub questions: Vec<QuestionBankQuestionView>,
    pub created_at: DateTime<Utc>,
}

impl From<QuestionBank> for QuestionBankView {
    fn from(value: QuestionBank) -> Self {
        Self {
            id: value.id.to_string(),
            course_id: value.course_id,
            name: value.name,
            questions: value
                .questions
                .into_iter()
                .map(QuestionBankQuestionView::from)
                .collect(),
            created_at: value.created_at,
        }
    }
}

impl From<QuestionBankQuestion> for QuestionBankQuestionView {
    fn from(value: QuestionBankQuestion) -> Self {
        Self {
            id: value.id,
            prompt: value.prompt,
            options: value.options,
            answer_index: value.answer_index,
            images: value.images,
        }
    }
}
