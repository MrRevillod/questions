use crate::{
    courses::CourseId,
    shared::{Entity, Id},
};

use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

pub type QuizId = Id<Quiz>;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow, Builder)]
pub struct Quiz {
    #[builder(default = QuizId::new())]
    pub id: QuizId,
    pub course_id: CourseId,
    pub title: String,
    pub kind: QuizKind,
    pub join_code: String,
    pub question_count: i16,
    pub certainty_table: Option<CertaintyTable>,
    pub attempt_duration_minutes: i16,
    pub starts_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Entity for Quiz {
    fn key_name() -> &'static str {
        "quiz"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "quiz_kind", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum QuizKind {
    Traditional,
    Certainty,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "certainty_score")]
pub struct CertaintyScore {
    pub correct: i16,
    pub incorrect: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "certainty_table")]
pub struct CertaintyTable {
    pub low: CertaintyScore,
    pub medium: CertaintyScore,
    pub high: CertaintyScore,
}
