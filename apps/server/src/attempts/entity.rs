use crate::quizzes::QuizParticipantView;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus {
    InProgress,
    Submitted,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "attempt_certainty_level", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AttemptCertaintyLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct AttemptEntity {
    pub id: Uuid,
    pub quiz_id: Uuid,
    pub student_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub question_order: Vec<Uuid>,
    pub score_points: Option<f64>,
    pub score_points_max: Option<f64>,
    pub grade: Option<f64>,
    pub evaluated_at: Option<DateTime<Utc>>,
    pub evaluated_by: Option<Uuid>,
    pub results_released_at: Option<DateTime<Utc>>,
    pub results_viewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct AttemptAnswerEntity {
    pub attempt_id: Uuid,
    pub question_id: Uuid,
    pub answer_index: i16,
    pub certainty_level: Option<AttemptCertaintyLevel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptAnswerView {
    pub question_id: Uuid,
    pub answer_index: i16,
    pub certainty_level: Option<AttemptCertaintyLevel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptSnapshotView {
    pub attempt_id: Uuid,
    pub quiz_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: AttemptStatus,
    pub quiz: QuizParticipantView,
    pub answers: Vec<AttemptAnswerView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptQuestionResultView {
    pub question_id: Uuid,
    pub question: String,
    pub options: Vec<String>,
    pub images: Vec<String>,
    pub answer_index: Option<i16>,
    pub correct_answer_index: i16,
    pub certainty_level: Option<AttemptCertaintyLevel>,
    pub is_correct: bool,
    pub awarded_points: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptResultView {
    pub attempt_id: Uuid,
    pub quiz_id: Uuid,
    pub status: AttemptStatus,
    pub submitted_at: DateTime<Utc>,
    pub evaluated_at: DateTime<Utc>,
    pub score_points: f64,
    pub score_points_max: f64,
    pub grade: f64,
    pub results_released_at: DateTime<Utc>,
    pub results_viewed_at: Option<DateTime<Utc>>,
    pub questions: Vec<AttemptQuestionResultView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ManagedAttemptSummaryView {
    pub attempt_id: Uuid,
    pub quiz_id: Uuid,
    pub student_id: Uuid,
    pub student_name: String,
    pub student_username: String,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub score_points: Option<f64>,
    pub score_points_max: Option<f64>,
    pub grade: Option<f64>,
    pub results_released_at: Option<DateTime<Utc>>,
    pub results_viewed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalizeAndPublishSummaryView {
    pub quiz_id: Uuid,
    pub finalized_attempts: usize,
    pub published_attempts: usize,
}

impl From<AttemptAnswerEntity> for AttemptAnswerView {
    fn from(answer: AttemptAnswerEntity) -> Self {
        Self {
            question_id: answer.question_id,
            answer_index: answer.answer_index,
            certainty_level: answer.certainty_level,
        }
    }
}
