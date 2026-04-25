use crate::{
    courses::CourseId,
    quizzes::QuizId,
    shared::{Entity, Id},
};

use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

pub type QuestionBankId = Id<QuestionBank>;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow, Builder)]
pub struct QuestionBank {
    #[builder(default = QuestionBankId::new())]
    pub id: QuestionBankId,
    pub course_id: CourseId,
    pub name: String,
    pub questions: Vec<QuestionBankQuestion>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Entity for QuestionBank {
    fn key_name() -> &'static str {
        "question_bank"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Type, Builder)]
#[sqlx(type_name = "question")]
pub struct QuestionBankQuestion {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub prompt: String,
    pub options: Vec<String>,
    pub answer_index: i16,
    pub images: Vec<String>,
}

#[derive(Clone, Debug, FromRow)]
pub struct LinkedQuiz {
    pub id: QuizId,
    pub question_count: i16,
    pub starts_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}
