use crate::quizzes::{Quiz, QuizError, QuizId, QuizRepository};
use crate::shared::AppResult;
use crate::users::{User, UserRole};

use sword::prelude::*;

#[injectable]
pub struct QuizPolicy {
    repository: QuizRepository,
}

impl QuizPolicy {
    pub fn can_create_quiz(&self, current_user: &User) -> AppResult<()> {
        if matches!(current_user.role, UserRole::Func | UserRole::Assistant) {
            return Ok(());
        }

        Err(QuizError::Forbidden)?
    }

    pub fn can_join_quiz(&self, current_user: &User) -> AppResult<()> {
        if current_user.role == UserRole::Student {
            return Ok(());
        }

        Err(QuizError::Forbidden)?
    }

    pub fn can_list_managed_quizzes(&self, current_user: &User) -> AppResult<()> {
        if matches!(current_user.role, UserRole::Func | UserRole::Assistant) {
            return Ok(());
        }

        Err(QuizError::Forbidden)?
    }

    pub async fn require_managed_quiz(
        &self,
        current_user: &User,
        quiz_id: &QuizId,
    ) -> AppResult<Quiz> {
        let Some(quiz) = self.repository.find_by_id(quiz_id).await? else {
            return Err(QuizError::NotFound(quiz_id.to_string()))?;
        };

        if !self
            .repository
            .has_course_access(quiz_id, &current_user.id)
            .await?
        {
            return Err(QuizError::Forbidden)?;
        }

        Ok(quiz)
    }
}
