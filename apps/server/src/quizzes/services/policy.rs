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
        if self.can_manage_quizzes(current_user) {
            return Ok(());
        }

        Err(QuizError::Forbidden)?
    }

    pub async fn can_read_managed_quiz(&self, current_user: &User, quiz: &Quiz) -> AppResult<()> {
        let is_collaborator = self
            .repository
            .is_collaborator(&quiz.id, &current_user.id)
            .await?;

        if self.has_managed_quiz_access(current_user, quiz, is_collaborator) {
            return Ok(());
        }

        Err(QuizError::Forbidden)?
    }

    pub async fn can_update_quiz(&self, current_user: &User, quiz: &Quiz) -> AppResult<()> {
        self.can_read_managed_quiz(current_user, quiz).await
    }

    pub fn can_join_quiz(&self, current_user: &User) -> AppResult<()> {
        if current_user.role == UserRole::Student {
            return Ok(());
        }

        Err(QuizError::Forbidden)?
    }

    pub fn can_manage_collaborators(&self, current_user: &User, quiz: &Quiz) -> AppResult<()> {
        if self.is_owner(current_user, quiz) {
            return Ok(());
        }

        Err(QuizError::OnlyOwnerCanManageCollaborators)?
    }

    pub async fn can_list_managed_quizzes(&self, current_user: &User) -> AppResult<()> {
        if self.can_manage_quizzes(current_user) {
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

        self.can_read_managed_quiz(current_user, &quiz).await?;

        Ok(quiz)
    }

    pub async fn require_owner_quiz(
        &self,
        current_user: &User,
        quiz_id: &QuizId,
    ) -> AppResult<Quiz> {
        let Some(quiz) = self.repository.find_by_id(quiz_id).await? else {
            return Err(QuizError::NotFound(quiz_id.to_string()))?;
        };

        self.can_manage_collaborators(current_user, &quiz)?;

        Ok(quiz)
    }

    fn can_manage_quizzes(&self, current_user: &User) -> bool {
        matches!(current_user.role, UserRole::Func | UserRole::Assistant)
    }

    fn is_owner(&self, current_user: &User, quiz: &Quiz) -> bool {
        quiz.owner_id == current_user.id
    }

    fn has_managed_quiz_access(
        &self,
        current_user: &User,
        quiz: &Quiz,
        is_collaborator: bool,
    ) -> bool {
        self.is_owner(current_user, quiz) || is_collaborator
    }
}
