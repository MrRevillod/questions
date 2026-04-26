use std::sync::Arc;

use crate::banks::{QuestionBankId, QuestionBankQuestion};
use crate::quizzes::{Quiz, QuizId};
use crate::shared::{AppResult, Tx};
use crate::snapshots::SnapshotRepository;

use sword::prelude::*;

#[injectable]
pub struct SnapshotService {
    repository: Arc<SnapshotRepository>,
}

impl SnapshotService {
    pub async fn list_linked_quizzes(&self, bank_id: &QuestionBankId) -> AppResult<Vec<Quiz>> {
        self.repository.list_linked_quizzes(bank_id).await
    }

    pub async fn list_questions_for_quiz(
        &self,
        quiz_id: &QuizId,
    ) -> AppResult<Vec<QuestionBankQuestion>> {
        self.repository.list_questions_for_quiz(quiz_id).await
    }

    pub async fn upsert_questions(
        &self,
        tx: &mut Tx<'_>,
        quiz_id: &QuizId,
        questions: &[QuestionBankQuestion],
    ) -> AppResult<()> {
        self.repository
            .upsert_questions(tx, quiz_id, questions)
            .await
    }

    pub async fn update_questions(
        &self,
        tx: &mut Tx<'_>,
        quiz_id: &QuizId,
        questions: &[QuestionBankQuestion],
    ) -> AppResult<bool> {
        self.repository
            .update_questions(tx, quiz_id, questions)
            .await
    }
}
