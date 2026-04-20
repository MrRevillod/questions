use crate::quizzes::{Quiz, QuizCollaborator};
use crate::shared::{AppResult, Database};
use crate::users::User;

use chrono::Utc;
use sword::prelude::*;
use uuid::Uuid;

#[injectable]
pub struct QuizRepository {
    db: Database,
}

impl QuizRepository {
    pub async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Quiz>> {
        let quiz = sqlx::query_as::<_, Quiz>("SELECT * FROM quizzes WHERE id = $1")
            .bind(id)
            .fetch_optional(self.db.get_pool())
            .await?;

        Ok(quiz)
    }

    pub async fn find_by_code(&self, code: &str) -> AppResult<Option<Quiz>> {
        let quiz = sqlx::query_as::<_, Quiz>("SELECT * FROM quizzes WHERE join_code = $1")
            .bind(code)
            .fetch_optional(self.db.get_pool())
            .await?;

        Ok(quiz)
    }

    pub async fn list_managed_by_user(&self, user_id: &Uuid) -> AppResult<Vec<Quiz>> {
        let quizzes = sqlx::query_as::<_, Quiz>(
            "SELECT q.*
             FROM quizzes q
             WHERE q.owner_id = $1
                OR EXISTS (
                    SELECT 1
                    FROM quiz_collaborators qc
                    WHERE qc.quiz_id = q.id AND qc.user_id = $1
                )
             ORDER BY q.created_at DESC",
        )
        .bind(user_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(quizzes)
    }

    pub async fn create(&self, quiz: Quiz) -> AppResult<Quiz> {
        let quiz = sqlx::query_as::<_, Quiz>(
            "INSERT INTO quizzes (
                id,
                owner_id,
                title,
                kind,
                join_code,
                questions,
                certainly_table,
                start_time,
                attempt_duration_minutes,
                question_count,
                closed_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *",
        )
        .bind(quiz.id)
        .bind(quiz.owner_id)
        .bind(&quiz.title)
        .bind(&quiz.kind)
        .bind(&quiz.join_code)
        .bind(&quiz.questions)
        .bind(&quiz.certainly_table)
        .bind(quiz.start_time)
        .bind(quiz.attempt_duration_minutes)
        .bind(quiz.question_count)
        .bind(quiz.closed_at)
        .bind(quiz.created_at)
        .bind(quiz.updated_at)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(quiz)
    }

    pub async fn update(&self, quiz: &Quiz) -> AppResult<Quiz> {
        let updated = sqlx::query_as::<_, Quiz>(
            "UPDATE quizzes
             SET title = $2,
                  questions = $3,
                  certainly_table = $4,
                  start_time = $5,
                  attempt_duration_minutes = $6,
                  question_count = $7,
                  closed_at = $8,
                  updated_at = $9
             WHERE id = $1
             RETURNING *",
        )
        .bind(quiz.id)
        .bind(&quiz.title)
        .bind(&quiz.questions)
        .bind(&quiz.certainly_table)
        .bind(quiz.start_time)
        .bind(quiz.attempt_duration_minutes)
        .bind(quiz.question_count)
        .bind(quiz.closed_at)
        .bind(quiz.updated_at)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(updated)
    }

    pub async fn has_attempts(&self, quiz_id: &Uuid) -> AppResult<bool> {
        let has_attempts = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1
                FROM quiz_attempts
                WHERE quiz_id = $1
            )",
        )
        .bind(quiz_id)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(has_attempts)
    }

    pub async fn add_collaborator(
        &self,
        quiz_id: &Uuid,
        user_id: &Uuid,
    ) -> AppResult<QuizCollaborator> {
        let collaborator = sqlx::query_as::<_, QuizCollaborator>(
            "INSERT INTO quiz_collaborators (quiz_id, user_id, created_at)
             VALUES ($1, $2, $3)
             ON CONFLICT (quiz_id, user_id) DO UPDATE
             SET user_id = EXCLUDED.user_id
             RETURNING *",
        )
        .bind(quiz_id)
        .bind(user_id)
        .bind(Utc::now())
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(collaborator)
    }

    pub async fn remove_collaborator(&self, quiz_id: &Uuid, user_id: &Uuid) -> AppResult<bool> {
        let result =
            sqlx::query("DELETE FROM quiz_collaborators WHERE quiz_id = $1 AND user_id = $2")
                .bind(quiz_id)
                .bind(user_id)
                .execute(self.db.get_pool())
                .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn is_collaborator(&self, quiz_id: &Uuid, user_id: &Uuid) -> AppResult<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1
                FROM quiz_collaborators
                WHERE quiz_id = $1 AND user_id = $2
            )",
        )
        .bind(quiz_id)
        .bind(user_id)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(exists)
    }

    pub async fn list_collaborator_users(&self, quiz_id: &Uuid) -> AppResult<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT u.*
             FROM users u
             INNER JOIN quiz_collaborators qc ON qc.user_id = u.id
             WHERE qc.quiz_id = $1
             ORDER BY u.username ASC",
        )
        .bind(quiz_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(users)
    }

    pub async fn close_quiz(&self, quiz_id: &Uuid) -> AppResult<Quiz> {
        let now = Utc::now();

        let quiz = sqlx::query_as::<_, Quiz>(
            "UPDATE quizzes
             SET closed_at = COALESCE(closed_at, $2),
                 updated_at = $2
             WHERE id = $1
             RETURNING *",
        )
        .bind(quiz_id)
        .bind(now)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(quiz)
    }

    pub async fn delete_by_id(&self, quiz_id: &Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM quizzes WHERE id = $1")
            .bind(quiz_id)
            .execute(self.db.get_pool())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
