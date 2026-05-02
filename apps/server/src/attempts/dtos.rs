use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use chrono::{DateTime, Utc};

use crate::{
    attempts::{Attempt, AttemptId},
    quizzes::{CertaintyLevel, QuizId},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus {
    InProgress,
    Submitted,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptSnapshotView {
    pub attempt_id: AttemptId,
    pub quiz_id: QuizId,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub status: AttemptStatus,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SaveAttemptAnswerDto {
    #[validate(range(min = 0, message = "answerIndex must be greater or equal than 0"))]
    pub answer_index: i16,
    pub certainty_level: Option<CertaintyLevel>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptQuestionResultView {
    pub question_id: Uuid,
    pub question: String,
    pub options: Vec<String>,
    pub images: Vec<String>,
    pub answer_index: Option<i16>,
    pub correct_answer_index: i16,
    pub certainty_level: Option<CertaintyLevel>,
    pub is_correct: bool,
    pub awarded_points: i16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptResultView {
    pub attempt_id: AttemptId,
    pub quiz_id: QuizId,
    pub status: AttemptStatus,
    pub submitted_at: DateTime<Utc>,
    pub evaluated_at: DateTime<Utc>,
    pub score_points: i16,
    pub score_points_max: i16,
    pub grade: f64,
    pub results_viewed_at: Option<DateTime<Utc>>,
    pub questions: Vec<AttemptQuestionResultView>,
}

impl From<&Attempt> for AttemptSnapshotView {
    fn from(value: &Attempt) -> Self {
        Self {
            attempt_id: value.id,
            quiz_id: value.quiz_id,
            started_at: value.started_at,
            expires_at: value.expires_at,
            submitted_at: value.submitted_at,
            status: if value.submitted_at.is_some() {
                AttemptStatus::Submitted
            } else {
                AttemptStatus::InProgress
            },
        }
    }
}
