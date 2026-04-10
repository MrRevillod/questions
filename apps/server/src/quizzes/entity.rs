use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "quiz_kind")]
pub enum QuizKind {
    Traditional,
    Certainly,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "certainly_level")]
pub struct CertainlyLevel {
    pub correct: i16,
    pub incorrect: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "certainly_table")]
pub struct CertainlyTable {
    pub low: CertainlyLevel,
    pub medium: CertainlyLevel,
    pub high: CertainlyLevel,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "question")]
pub struct QuizQuestion {
    pub id: Uuid,
    pub question: String,
    pub options: Vec<String>,
    pub answer: i16,
    pub images: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct QuizEntity {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub kind: QuizKind,
    pub join_code: String,
    pub questions: Vec<QuizQuestion>,
    pub certainly_table: Option<CertainlyTable>,
    pub start_time: DateTime<Utc>,
    pub attempt_duration_minutes: i32,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct QuizCollaboratorEntity {
    pub quiz_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}
