use crate::banks::{Question, QuestionBank, QuestionBankId};
use crate::courses::CourseId;
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

    pub async fn save(&self, tx: &mut Tx<'_>, bank: &QuestionBank) -> AppResult<QuestionBank> {
        let bank = sqlx::query_as::<_, QuestionBank>(
            "INSERT INTO question_banks (id, course_id, name, questions, created_at, deleted_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id)
             DO UPDATE SET
                 course_id = EXCLUDED.course_id,
                 name = EXCLUDED.name,
                 questions = EXCLUDED.questions,
                 created_at = EXCLUDED.created_at,
                 deleted_at = EXCLUDED.deleted_at
             RETURNING *",
        )
        .bind(bank.id)
        .bind(bank.course_id)
        .bind(&bank.name)
        .bind(&bank.questions)
        .bind(bank.created_at)
        .bind(bank.deleted_at)
        .fetch_one(&mut **tx)
        .await?;

        Ok(bank)
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

    pub async fn are_banks_in_course(
        &self,
        bank_ids: &[QuestionBankId],
        course_id: &CourseId,
    ) -> AppResult<bool> {
        if bank_ids.is_empty() {
            return Ok(false);
        }

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM question_banks
            WHERE id = ANY($1) AND course_id = $2 AND deleted_at IS NULL",
        )
        .bind(bank_ids)
        .bind(course_id)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(count as usize == bank_ids.len())
    }

    pub async fn list_questions_by_bank_ids(
        &self,
        bank_ids: &[QuestionBankId],
    ) -> AppResult<Vec<Question>> {
        let rows = sqlx::query_scalar::<_, Vec<Question>>(
            "SELECT qb.questions FROM question_banks qb
             WHERE qb.id = ANY($1) AND qb.deleted_at IS NULL",
        )
        .bind(bank_ids)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(rows.into_iter().flatten().collect())
    }
}
