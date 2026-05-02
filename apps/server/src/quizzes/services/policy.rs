use std::sync::Arc;

use crate::courses::{CourseId, CourseRepository};
use crate::quizzes::{Quiz, QuizError, QuizId, QuizRepository};
use crate::shared::AppResult;
use crate::users::{User, UserRole};

use sword::prelude::*;

#[injectable]
pub struct QuizPolicy {
    quizzes: Arc<QuizRepository>,
    courses: Arc<CourseRepository>,
}

impl QuizPolicy {
    pub async fn check_can_create_quiz(
        &self,
        current_user: &User,
        course_id: &CourseId,
    ) -> AppResult<bool> {
        if current_user.role == UserRole::Admin {
            return Ok(true);
        }

        if !self.courses.is_member(course_id, &current_user.id).await? {
            return Err(QuizError::Forbidden)?;
        }

        Ok(true)
    }

    pub async fn require_managed_quiz(
        &self,
        current_user: &User,
        quiz_id: &QuizId,
    ) -> AppResult<Quiz> {
        let Some(quiz) = self.quizzes.find_by_id(quiz_id).await? else {
            return Err(QuizError::NotFound(quiz_id.to_string()))?;
        };

        if current_user.role == UserRole::Admin {
            return Ok(quiz);
        }

        if !self
            .courses
            .is_member(&quiz.course_id, &current_user.id)
            .await?
        {
            return Err(QuizError::Forbidden)?;
        }

        Ok(quiz)
    }
}
