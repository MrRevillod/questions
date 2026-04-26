use crate::banks::QuestionBankQuestion;
use crate::quizzes::QuizId;

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow)]
#[allow(dead_code)]
pub struct QuestionBankSnapshot {
    pub id: Uuid,
    pub quiz_id: QuizId,
    pub questions: Vec<QuestionBankQuestion>,
    pub deleted_at: Option<DateTime<Utc>>,
}
