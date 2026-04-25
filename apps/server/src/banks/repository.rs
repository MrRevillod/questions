use crate::banks::{LinkedQuiz, QuestionBank, QuestionBankId, QuestionBankQuestion};
use crate::courses::CourseId;
use crate::quizzes::QuizId;
use crate::shared::{AppResult, Database, Tx};

use chrono::Utc;
use std::sync::Arc;
use sword::prelude::*;

#[injectable]
pub struct QuestionBankRepository {
    db: Arc<Database>,
}

impl QuestionBankRepository {
    pub async fn find_by_id(&self, bank_id: &QuestionBankId) -> AppResult<Option<QuestionBank>> {
        let bank = sqlx::query_as::<_, QuestionBank>(
            "SELECT * FROM question_banks WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(bank_id)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(bank)
    }

    pub async fn list_by_course(&self, course_id: &CourseId) -> AppResult<Vec<QuestionBank>> {
        let banks = sqlx::query_as::<_, QuestionBank>(
            "SELECT * FROM question_banks
             WHERE course_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(course_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(banks)
    }

    pub async fn create(&self, tx: &mut Tx<'_>, bank: &QuestionBank) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO question_banks (id, course_id, name, questions, created_at, deleted_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(bank.id)
        .bind(bank.course_id)
        .bind(&bank.name)
        .bind(&bank.questions)
        .bind(bank.created_at)
        .bind(bank.deleted_at)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn update(&self, tx: &mut Tx<'_>, bank: &QuestionBank) -> AppResult<()> {
        sqlx::query("UPDATE question_banks SET name = $2, questions = $3 WHERE id = $1")
            .bind(bank.id)
            .bind(&bank.name)
            .bind(&bank.questions)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    pub async fn soft_delete(&self, tx: &mut Tx<'_>, bank_id: &QuestionBankId) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE question_banks SET deleted_at = $2
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(bank_id)
        .bind(Utc::now())
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_linked_quizzes(
        &self,
        bank_id: &QuestionBankId,
    ) -> AppResult<Vec<LinkedQuiz>> {
        let quizzes = sqlx::query_as::<_, LinkedQuiz>(
            "SELECT q.id, q.question_count, q.starts_at, q.closed_at
             FROM quizzes q
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

        let questions = rows.into_iter().flatten().collect();

        Ok(questions)
    }

    pub async fn update_snapshot_questions(
        &self,
        tx: &mut Tx<'_>,
        quiz_id: &QuizId,
        questions: &[QuestionBankQuestion],
    ) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE question_bank_snapshots
             SET questions = $2
             WHERE quiz_id = $1 AND deleted_at IS NULL",
        )
        .bind(quiz_id)
        .bind(questions)
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
