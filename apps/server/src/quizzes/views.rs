use crate::quizzes::{CertainlyTable, Quiz, QuizKind, QuizQuestion};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizSummaryView {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub kind: QuizKind,
    pub join_code: String,
    pub question_count: usize,
    pub start_time: DateTime<Utc>,
    pub attempt_duration_minutes: i32,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizQuestionView {
    pub question_id: Uuid,
    pub question: String,
    pub options: Vec<String>,
    pub images: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizDetailView {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinQuizPreviewView {
    pub id: Uuid,
    pub title: String,
    pub kind: QuizKind,
    pub question_count: usize,
    pub certainly_table: Option<CertainlyTable>,
    pub start_time: DateTime<Utc>,
    pub attempt_duration_minutes: i32,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizParticipantView {
    pub id: Uuid,
    pub title: String,
    pub kind: QuizKind,
    pub questions: Vec<QuizQuestionView>,
    pub certainly_table: Option<CertainlyTable>,
    pub start_time: DateTime<Utc>,
    pub attempt_duration_minutes: i32,
    pub closed_at: Option<DateTime<Utc>>,
}

impl From<Quiz> for QuizSummaryView {
    fn from(quiz: Quiz) -> Self {
        Self {
            id: quiz.id,
            owner_id: quiz.owner_id,
            title: quiz.title,
            kind: quiz.kind,
            join_code: quiz.join_code,
            question_count: quiz.question_count as usize,
            start_time: quiz.start_time,
            attempt_duration_minutes: quiz.attempt_duration_minutes,
            closed_at: quiz.closed_at,
            created_at: quiz.created_at,
        }
    }
}

impl From<Quiz> for QuizDetailView {
    fn from(quiz: Quiz) -> Self {
        Self {
            id: quiz.id,
            owner_id: quiz.owner_id,
            title: quiz.title,
            kind: quiz.kind,
            join_code: quiz.join_code,
            questions: quiz.questions,
            certainly_table: quiz.certainly_table,
            start_time: quiz.start_time,
            attempt_duration_minutes: quiz.attempt_duration_minutes,
            closed_at: quiz.closed_at,
            created_at: quiz.created_at,
            updated_at: quiz.updated_at,
        }
    }
}

impl From<&Quiz> for JoinQuizPreviewView {
    fn from(quiz: &Quiz) -> Self {
        Self {
            id: quiz.id,
            title: quiz.title.clone(),
            kind: quiz.kind.clone(),
            question_count: quiz.question_count as usize,
            certainly_table: quiz.certainly_table.clone(),
            start_time: quiz.start_time,
            attempt_duration_minutes: quiz.attempt_duration_minutes,
            closed_at: quiz.closed_at,
        }
    }
}

impl From<QuizQuestion> for QuizQuestionView {
    fn from(question: QuizQuestion) -> Self {
        Self {
            question_id: question.id,
            question: question.question,
            options: question.options,
            images: question.images,
        }
    }
}

impl From<Quiz> for QuizParticipantView {
    fn from(quiz: Quiz) -> Self {
        Self {
            id: quiz.id,
            title: quiz.title,
            kind: quiz.kind,
            questions: quiz
                .questions
                .into_iter()
                .map(QuizQuestionView::from)
                .collect(),
            certainly_table: quiz.certainly_table,
            start_time: quiz.start_time,
            attempt_duration_minutes: quiz.attempt_duration_minutes,
            closed_at: quiz.closed_at,
        }
    }
}
