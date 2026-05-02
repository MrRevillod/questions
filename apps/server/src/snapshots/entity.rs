use crate::banks::Question;

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow)]
#[allow(dead_code)]
pub struct QuestionBankSnapshot {
    pub id: Uuid,
    pub questions: Vec<Question>,
    pub deleted_at: Option<DateTime<Utc>>,
}
