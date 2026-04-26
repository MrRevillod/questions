use std::sync::Arc;

use crate::banks::{QuestionBankId, QuestionBankQuestion};
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

    pub async fn list_questions_for_quiz(
        &self,
        quiz_id: &QuizId,
    ) -> AppResult<Vec<QuestionBankQuestion>> {
        let rows = sqlx::query_scalar::<_, Vec<QuestionBankQuestion>>(
            "SELECT qb.questions FROM question_banks qb
             INNER JOIN quiz_question_banks qqb ON qqb.question_bank_id = qb.id
             WHERE qqb.quiz_id = $1 AND qb.deleted_at IS NULL",
        )
        .bind(quiz_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(rows.into_iter().flatten().collect())
    }

    pub async fn upsert_questions(
        &self,
        tx: &mut Tx<'_>,
        quiz_id: &QuizId,
        questions: &[QuestionBankQuestion],
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO question_bank_snapshots (id, quiz_id, questions, deleted_at)
             VALUES ($1, $2, $3, NULL)
             ON CONFLICT (quiz_id) WHERE deleted_at IS NULL
             DO UPDATE SET questions = EXCLUDED.questions",
        )
        .bind(Uuid::new_v4())
        .bind(quiz_id)
        .bind(questions)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn update_questions(
        &self,
        tx: &mut Tx<'_>,
        quiz_id: &QuizId,
        questions: &[QuestionBankQuestion],
    ) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE question_bank_snapshots SET questions = $2
             WHERE quiz_id = $1 AND deleted_at IS NULL",
        )
        .bind(quiz_id)
        .bind(questions)
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
