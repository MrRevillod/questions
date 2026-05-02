use std::sync::Arc;

use crate::banks::{Question, QuestionBankId};
use crate::quizzes::{Quiz, QuizId};
use crate::shared::{AppResult, Tx};
use crate::snapshots::SnapshotRepository;

use sword::prelude::*;
use uuid::Uuid;

#[injectable]
pub struct SnapshotService {
    repository: Arc<SnapshotRepository>,
}

impl SnapshotService {
    pub async fn list_linked_quizzes(&self, bank_id: &QuestionBankId) -> AppResult<Vec<Quiz>> {
        self.repository.list_linked_quizzes(bank_id).await
    }

    pub async fn list_questions_for_linked_banks(
        &self,
        quiz_id: &QuizId,
    ) -> AppResult<Vec<Question>> {
        self.repository
            .list_questions_for_linked_banks(quiz_id)
            .await
    }

    pub async fn create_snapshot(
        &self,
        tx: &mut Tx<'_>,
        snapshot_id: Uuid,
        questions: &[Question],
    ) -> AppResult<()> {
        self.repository
            .create_snapshot(tx, snapshot_id, questions)
            .await
    }

    pub async fn update_questions(
        &self,
        tx: &mut Tx<'_>,
        snapshot_id: Uuid,
        questions: &[Question],
    ) -> AppResult<bool> {
        self.repository
            .update_questions(tx, snapshot_id, questions)
            .await
    }
}
