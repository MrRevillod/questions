use std::sync::Arc;

use crate::banks::{Question, QuestionBankId};
use crate::quizzes::{Quiz, QuizId};
use crate::shared::{AppResult, Database, Tx};

use sword::prelude::*;
use uuid::Uuid;

#[injectable]
pub struct SnapshotRepository {
    db: Arc<Database>,
}

impl SnapshotRepository {
    pub async fn list_linked_quizzes(&self, bank_id: &QuestionBankId) -> AppResult<Vec<Quiz>> {
        let quizzes = sqlx::query_as::<_, Quiz>(
            "SELECT q.* FROM quizzes q
             INNER JOIN quiz_question_banks qqb ON qqb.quiz_id = q.id
             WHERE qqb.question_bank_id = $1 AND q.deleted_at IS NULL",
        )
        .bind(bank_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(quizzes)
    }

    pub async fn list_questions_for_quiz(&self, quiz_id: &QuizId) -> AppResult<Vec<Question>> {
        let questions = sqlx::query_scalar::<_, Vec<Question>>(
            "SELECT qbs.questions FROM quizzes q
             INNER JOIN question_bank_snapshots qbs ON qbs.id = q.snapshot_id
             WHERE q.id = $1 AND q.deleted_at IS NULL",
        )
        .bind(quiz_id)
        .fetch_optional(self.db.get_pool())
        .await?
        .unwrap_or_default();

        Ok(questions)
    }

    pub async fn list_questions_for_linked_banks(
        &self,
        quiz_id: &QuizId,
    ) -> AppResult<Vec<Question>> {
        let rows = sqlx::query_scalar::<_, Vec<Question>>(
            "SELECT qb.questions FROM question_banks qb
             INNER JOIN quiz_question_banks qqb ON qqb.question_bank_id = qb.id
             WHERE qqb.quiz_id = $1 AND qb.deleted_at IS NULL",
        )
        .bind(quiz_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(rows.into_iter().flatten().collect())
    }

    pub async fn create_snapshot(
        &self,
        tx: &mut Tx<'_>,
        snapshot_id: Uuid,
        questions: &[Question],
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO question_bank_snapshots (id, questions, deleted_at)
             VALUES ($1, $2, NULL)",
        )
        .bind(snapshot_id)
        .bind(questions)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn update_questions(
        &self,
        tx: &mut Tx<'_>,
        snapshot_id: Uuid,
        questions: &[Question],
    ) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE question_bank_snapshots SET questions = $2
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(snapshot_id)
        .bind(questions)
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
