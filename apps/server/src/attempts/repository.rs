use crate::attempts::{AttemptAnswerEntity, AttemptEntity, ManagedAttemptSummaryView};
use crate::shared::{AppResult, Database};

use chrono::Utc;
use sword::prelude::*;
use uuid::Uuid;

#[injectable]
pub struct AttemptRepository {
    db: Database,
}

impl AttemptRepository {
    pub async fn find_by_id(&self, attempt_id: &Uuid) -> AppResult<Option<AttemptEntity>> {
        let attempt =
            sqlx::query_as::<_, AttemptEntity>("SELECT * FROM quiz_attempts WHERE id = $1")
                .bind(attempt_id)
                .fetch_optional(self.db.get_pool())
                .await?;

        Ok(attempt)
    }

    pub async fn find_active_for_quiz(
        &self,
        quiz_id: &Uuid,
        student_id: &Uuid,
    ) -> AppResult<Option<AttemptEntity>> {
        let attempt = sqlx::query_as::<_, AttemptEntity>(
            "SELECT *
             FROM quiz_attempts
             WHERE quiz_id = $1 AND student_id = $2 AND submitted_at IS NULL",
        )
        .bind(quiz_id)
        .bind(student_id)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(attempt)
    }

    pub async fn find_by_quiz_and_student(
        &self,
        quiz_id: &Uuid,
        student_id: &Uuid,
    ) -> AppResult<Option<AttemptEntity>> {
        let attempt = sqlx::query_as::<_, AttemptEntity>(
            "SELECT *
             FROM quiz_attempts
             WHERE quiz_id = $1 AND student_id = $2",
        )
        .bind(quiz_id)
        .bind(student_id)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(attempt)
    }

    pub async fn list_for_quiz(&self, quiz_id: &Uuid) -> AppResult<Vec<ManagedAttemptSummaryView>> {
        let attempts = sqlx::query_as::<_, ManagedAttemptSummaryView>(
            "SELECT
                qa.id AS attempt_id,
                qa.quiz_id,
                qa.student_id,
                u.name AS student_name,
                u.username AS student_username,
                qa.started_at,
                qa.expires_at,
                qa.submitted_at,
                qa.score_points,
                qa.score_points_max,
                qa.grade,
                qa.results_released_at,
                qa.results_viewed_at
             FROM quiz_attempts qa
             INNER JOIN users u ON u.id = qa.student_id
             WHERE qa.quiz_id = $1
             ORDER BY qa.started_at DESC",
        )
        .bind(quiz_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(attempts)
    }

    pub async fn create(&self, attempt: AttemptEntity) -> AppResult<AttemptEntity> {
        let attempt = sqlx::query_as::<_, AttemptEntity>(
            "INSERT INTO quiz_attempts (
                id,
                quiz_id,
                student_id,
                started_at,
                expires_at,
                submitted_at,
                question_order,
                results_released_at,
                results_viewed_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *",
        )
        .bind(&attempt.id)
        .bind(&attempt.quiz_id)
        .bind(&attempt.student_id)
        .bind(&attempt.started_at)
        .bind(&attempt.expires_at)
        .bind(&attempt.submitted_at)
        .bind(&attempt.question_order)
        .bind(&attempt.results_released_at)
        .bind(&attempt.results_viewed_at)
        .bind(&attempt.updated_at)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(attempt)
    }

    pub async fn list_answers(&self, attempt_id: &Uuid) -> AppResult<Vec<AttemptAnswerEntity>> {
        let answers = sqlx::query_as::<_, AttemptAnswerEntity>(
            "SELECT * FROM quiz_answers WHERE attempt_id = $1",
        )
        .bind(attempt_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(answers)
    }

    pub async fn upsert_answer(
        &self,
        answer: AttemptAnswerEntity,
    ) -> AppResult<AttemptAnswerEntity> {
        let answer = sqlx::query_as::<_, AttemptAnswerEntity>(
            "INSERT INTO quiz_answers (
                attempt_id,
                question_id,
                answer_index,
                certainty_level
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (attempt_id, question_id) DO UPDATE
            SET answer_index = EXCLUDED.answer_index,
                certainty_level = EXCLUDED.certainty_level
            RETURNING *",
        )
        .bind(&answer.attempt_id)
        .bind(&answer.question_id)
        .bind(&answer.answer_index)
        .bind(&answer.certainty_level)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(answer)
    }

    pub async fn submit(&self, attempt_id: &Uuid) -> AppResult<AttemptEntity> {
        let now = Utc::now();

        let attempt = sqlx::query_as::<_, AttemptEntity>(
            "UPDATE quiz_attempts
             SET submitted_at = $2,
                  updated_at = $2
             WHERE id = $1 AND submitted_at IS NULL
             RETURNING *",
        )
        .bind(attempt_id)
        .bind(now)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(attempt)
    }

    pub async fn evaluate(
        &self,
        attempt_id: &Uuid,
        score_points: f64,
        score_points_max: f64,
        grade: f64,
        evaluated_by: Option<Uuid>,
    ) -> AppResult<AttemptEntity> {
        let now = Utc::now();

        let attempt = sqlx::query_as::<_, AttemptEntity>(
            "UPDATE quiz_attempts
             SET score_points = $2,
                 score_points_max = $3,
                 grade = $4,
                 evaluated_at = $5,
                 evaluated_by = $6,
                 updated_at = $5
             WHERE id = $1
             RETURNING *",
        )
        .bind(attempt_id)
        .bind(score_points)
        .bind(score_points_max)
        .bind(grade)
        .bind(now)
        .bind(evaluated_by)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(attempt)
    }

    pub async fn release_results_for_quiz(&self, quiz_id: &Uuid) -> AppResult<u64> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE quiz_attempts
             SET results_released_at = COALESCE(results_released_at, $2),
                 updated_at = $2
             WHERE quiz_id = $1
               AND submitted_at IS NOT NULL",
        )
        .bind(quiz_id)
        .bind(now)
        .execute(self.db.get_pool())
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn list_in_progress_for_quiz(&self, quiz_id: &Uuid) -> AppResult<Vec<AttemptEntity>> {
        let attempts = sqlx::query_as::<_, AttemptEntity>(
            "SELECT *
             FROM quiz_attempts
             WHERE quiz_id = $1
               AND submitted_at IS NULL",
        )
        .bind(quiz_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(attempts)
    }

    pub async fn list_submitted_for_quiz(&self, quiz_id: &Uuid) -> AppResult<Vec<AttemptEntity>> {
        let attempts = sqlx::query_as::<_, AttemptEntity>(
            "SELECT *
             FROM quiz_attempts
             WHERE quiz_id = $1
               AND submitted_at IS NOT NULL",
        )
        .bind(quiz_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(attempts)
    }

    pub async fn mark_results_viewed_once(
        &self,
        attempt_id: &Uuid,
    ) -> AppResult<Option<AttemptEntity>> {
        let now = Utc::now();

        let attempt = sqlx::query_as::<_, AttemptEntity>(
            "UPDATE quiz_attempts
             SET results_viewed_at = $2,
                 updated_at = $2
             WHERE id = $1
               AND submitted_at IS NOT NULL
               AND results_released_at IS NOT NULL
               AND results_viewed_at IS NULL
             RETURNING *",
        )
        .bind(attempt_id)
        .bind(now)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(attempt)
    }
}
