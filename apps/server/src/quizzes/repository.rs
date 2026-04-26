use std::sync::Arc;

use crate::banks::QuestionBankId;
use crate::quizzes::{Quiz, QuizId};
use crate::shared::{AppResult, Database, Tx};
use crate::users::UserId;

use chrono::Utc;
use sword::prelude::*;

#[injectable]
pub struct QuizRepository {
    db: Arc<Database>,
}

impl QuizRepository {
    pub async fn find_by_id(&self, id: &QuizId) -> AppResult<Option<Quiz>> {
        let quiz = sqlx::query_as::<_, Quiz>(
            "SELECT * FROM quizzes
            WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(quiz)
    }

    pub async fn find_by_code(&self, code: &str) -> AppResult<Option<Quiz>> {
        let quiz = sqlx::query_as::<_, Quiz>(
            "SELECT * FROM quizzes WHERE join_code = $1 AND deleted_at IS NULL",
        )
        .bind(code)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(quiz)
    }

    pub async fn list_managed_by_user(&self, user_id: &UserId) -> AppResult<Vec<Quiz>> {
        let quizzes = sqlx::query_as::<_, Quiz>(
            "SELECT q.* FROM quizzes q
             INNER JOIN course_members cm ON cm.course_id = q.course_id
             WHERE cm.user_id = $1 AND q.deleted_at IS NULL
             ORDER BY q.created_at DESC",
        )
        .bind(user_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(quizzes)
    }

    pub async fn has_course_access(&self, quiz_id: &QuizId, user_id: &UserId) -> AppResult<bool> {
        let has_access = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1
                FROM quizzes q
                INNER JOIN course_members cm ON cm.course_id = q.course_id
                WHERE q.id = $1
                  AND q.deleted_at IS NULL
                  AND cm.user_id = $2
            )",
        )
        .bind(quiz_id)
        .bind(user_id)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(has_access)
    }

    pub async fn save(&self, tx: &mut Tx<'_>, quiz: &Quiz) -> AppResult<Quiz> {
        let quiz = sqlx::query_as::<_, Quiz>(
            "INSERT INTO quizzes (
                id,
                course_id,
                title,
                kind,
                join_code,
                question_count,
                certainty_table,
                attempt_duration_minutes,
                starts_at,
                closed_at,
                created_at,
                deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id)
            DO UPDATE SET
                course_id = EXCLUDED.course_id,
                title = EXCLUDED.title,
                kind = EXCLUDED.kind,
                join_code = EXCLUDED.join_code,
                question_count = EXCLUDED.question_count,
                certainty_table = EXCLUDED.certainty_table,
                attempt_duration_minutes = EXCLUDED.attempt_duration_minutes,
                starts_at = EXCLUDED.starts_at,
                closed_at = EXCLUDED.closed_at,
                created_at = EXCLUDED.created_at,
                deleted_at = EXCLUDED.deleted_at
            RETURNING *",
        )
        .bind(quiz.id)
        .bind(quiz.course_id)
        .bind(&quiz.title)
        .bind(&quiz.kind)
        .bind(&quiz.join_code)
        .bind(quiz.question_count)
        .bind(&quiz.certainty_table)
        .bind(quiz.attempt_duration_minutes)
        .bind(quiz.starts_at)
        .bind(quiz.closed_at)
        .bind(quiz.created_at)
        .bind(quiz.deleted_at)
        .fetch_one(&mut **tx)
        .await?;

        Ok(quiz)
    }

    pub async fn close_quiz(&self, quiz_id: &QuizId) -> AppResult<()> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE quizzes
             SET closed_at = COALESCE(closed_at, $2)
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(quiz_id)
        .bind(now)
        .execute(self.db.get_pool())
        .await?;

        Ok(())
    }

    pub async fn delete_by_id(&self, quiz_id: &QuizId) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE quizzes SET deleted_at = $2
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(quiz_id)
        .bind(Utc::now())
        .execute(self.db.get_pool())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn set_bank_links(
        &self,
        tx: &mut Tx<'_>,
        quiz_id: &QuizId,
        bank_ids: &[QuestionBankId],
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM quiz_question_banks WHERE quiz_id = $1")
            .bind(quiz_id)
            .execute(&mut **tx)
            .await?;

        for bank_id in bank_ids {
            sqlx::query(
                "INSERT INTO quiz_question_banks (quiz_id, question_bank_id) VALUES ($1, $2)",
            )
            .bind(quiz_id)
            .bind(bank_id)
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }
}
