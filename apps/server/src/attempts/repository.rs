use std::sync::Arc;

use sqlx::{Postgres, QueryBuilder};
use sword::prelude::*;

use crate::{
    attempts::{Attempt, AttemptAnswer, AttemptFilter, AttemptId},
    quizzes::QuizId,
    shared::{AppResult, Database},
    users::UserId,
};

#[injectable]
pub struct AttemptRepository {
    db: Arc<Database>,
}

impl AttemptRepository {
    pub async fn find_by_id(&self, attempt_id: &AttemptId) -> AppResult<Option<Attempt>> {
        let attempt = sqlx::query_as::<_, Attempt>(
            "SELECT * FROM attempts WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(attempt_id)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(attempt)
    }

    pub async fn find_by_quiz_and_student(
        &self,
        quiz_id: &QuizId,
        student_id: &UserId,
    ) -> AppResult<Option<Attempt>> {
        let attempt = sqlx::query_as::<_, Attempt>(
            "SELECT * FROM attempts
             WHERE quiz_id = $1 AND student_id = $2 AND deleted_at IS NULL",
        )
        .bind(quiz_id)
        .bind(student_id)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(attempt)
    }

    pub async fn list_attempts(&self, filter: AttemptFilter) -> AppResult<Vec<Attempt>> {
        let mut query =
            QueryBuilder::<Postgres>::new("SELECT * FROM attempts WHERE deleted_at IS NULL");

        query.push(" AND quiz_id = ").push_bind(filter.quiz_id);

        if let Some(student_id) = filter.student_id {
            query.push(" AND student_id = ").push_bind(student_id);
        }

        let attempts = query
            .build_query_as::<Attempt>()
            .fetch_all(self.db.get_pool())
            .await?;

        Ok(attempts)
    }

    pub async fn save(&self, attempt: &Attempt) -> AppResult<()> {
        sqlx::query(
			"INSERT INTO attempts (id, student_id, quiz_id, question_order, score, grade, started_at, expires_at, submitted_at, results_viewed_at, deleted_at)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
			ON CONFLICT (id) DO UPDATE SET
				score = EXCLUDED.score,
				grade = EXCLUDED.grade,
				submitted_at = EXCLUDED.submitted_at,
				results_viewed_at = EXCLUDED.results_viewed_at,
				deleted_at = EXCLUDED.deleted_at
			"
		)
		.bind(attempt.id)
		.bind(attempt.student_id)
		.bind(attempt.quiz_id)
		.bind(&attempt.question_order)
		.bind(attempt.score)
		.bind(attempt.grade)
		.bind(attempt.started_at)
		.bind(attempt.expires_at)
		.bind(attempt.submitted_at)
		.bind(attempt.results_viewed_at)
		.bind(attempt.deleted_at)
		.execute(self.db.get_pool())
		.await?;

        Ok(())
    }

    pub async fn list_attempt_answers(
        &self,
        attempt_id: &AttemptId,
    ) -> AppResult<Vec<AttemptAnswer>> {
        let answers = sqlx::query_as::<_, AttemptAnswer>(
            "SELECT * FROM attempt_answers WHERE attempt_id = $1",
        )
        .bind(attempt_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(answers)
    }

    pub async fn upsert_attempt_answer(&self, answer: &AttemptAnswer) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO attempt_answers (attempt_id, question_id, answer_index, certainty_level)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (attempt_id, question_id)
             DO UPDATE SET
                answer_index = EXCLUDED.answer_index,
                certainty_level = EXCLUDED.certainty_level",
        )
        .bind(answer.attempt_id)
        .bind(answer.question_id)
        .bind(answer.answer_index)
        .bind(answer.certainty_level.clone())
        .execute(self.db.get_pool())
        .await?;

        Ok(())
    }

    pub async fn mark_results_viewed(&self, attempt_id: &AttemptId) -> AppResult<()> {
        sqlx::query(
            "UPDATE attempts SET results_viewed_at = COALESCE(results_viewed_at, NOW())
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(attempt_id)
        .execute(self.db.get_pool())
        .await?;

        Ok(())
    }
}
