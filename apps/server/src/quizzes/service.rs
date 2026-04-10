use crate::quizzes::{
    AddCollaboratorRequest, CreateQuizRequest, JoinQuizPreviewView, QuizDetailView, QuizError,
    QuizPolicy, QuizRepository, QuizSummaryView, UpdateQuizRequest,
};
use crate::shared::AppResult;
use crate::users::{User, UserRepository, UserRole};

use rand::RngExt;
use rand::distr::Alphanumeric;
use sword::prelude::*;
use uuid::Uuid;

#[injectable]
pub struct QuizService {
    policy: QuizPolicy,
    repository: QuizRepository,
    users: UserRepository,
}

impl QuizService {
    const JOIN_CODE_LENGTH: usize = 8;
    const JOIN_CODE_MAX_ATTEMPTS: usize = 10;

    pub async fn get_detail(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
    ) -> AppResult<QuizDetailView> {
        let quiz = self
            .policy
            .require_managed_quiz(current_user, quiz_id)
            .await?;

        Ok(QuizDetailView::from(quiz))
    }

    pub async fn list_managed_by_user(
        &self,
        current_user: &User,
    ) -> AppResult<Vec<QuizSummaryView>> {
        self.policy.can_list_managed_quizzes(current_user).await?;

        let quizzes = self
            .repository
            .list_managed_by_user(&current_user.id)
            .await?;

        Ok(quizzes.into_iter().map(QuizSummaryView::from).collect())
    }

    pub async fn get_join_preview(
        &self,
        current_user: &User,
        code: &str,
    ) -> AppResult<JoinQuizPreviewView> {
        self.policy.can_join_quiz(current_user)?;

        let Some(quiz) = self.repository.find_by_code(code).await? else {
            return Err(QuizError::NotFound(code.to_string()))?;
        };

        if quiz.closed_at.is_some() {
            return Err(QuizError::Closed.into());
        }

        Ok(JoinQuizPreviewView::from(&quiz))
    }

    pub async fn resolve_by_code_for_results(
        &self,
        current_user: &User,
        code: &str,
    ) -> AppResult<Uuid> {
        self.policy.can_join_quiz(current_user)?;

        let Some(quiz) = self.repository.find_by_code(code).await? else {
            return Err(QuizError::NotFound(code.to_string()))?;
        };

        Ok(quiz.id)
    }

    pub async fn create(
        &self,
        current_user: &User,
        input: CreateQuizRequest,
        owner_id: Uuid,
    ) -> AppResult<QuizDetailView> {
        self.policy.can_create_quiz(current_user)?;
        self.validate_collaborators(&input.collaborator_ids, &owner_id)
            .await?;
        let collaborator_ids = input.collaborator_ids.clone();
        let join_code = self.generate_unique_join_code().await?;
        let quiz = input.into_entity(owner_id, join_code);
        let quiz = self.repository.create(quiz).await?;

        for collaborator_id in collaborator_ids {
            if collaborator_id == owner_id {
                continue;
            }

            self.repository
                .add_collaborator(&quiz.id, &collaborator_id)
                .await?;
        }

        Ok(QuizDetailView::from(quiz))
    }

    pub async fn update(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
        input: UpdateQuizRequest,
    ) -> AppResult<QuizDetailView> {
        let mut quiz = self
            .policy
            .require_managed_quiz(current_user, quiz_id)
            .await?;
        self.policy.can_update_quiz(current_user, &quiz).await?;
        input.apply_to_entity(&mut quiz);

        let quiz = self.repository.update(&quiz).await?;

        Ok(QuizDetailView::from(quiz))
    }

    pub async fn add_collaborator(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
        input: AddCollaboratorRequest,
    ) -> AppResult<()> {
        let quiz = self
            .policy
            .require_owner_quiz(current_user, quiz_id)
            .await?;

        if input.user_id == quiz.owner_id {
            return Ok(());
        }

        if self
            .repository
            .is_collaborator(&quiz.id, &input.user_id)
            .await?
        {
            return Err(QuizError::CollaboratorAlreadyExists)?;
        }

        self.repository
            .add_collaborator(&quiz.id, &input.user_id)
            .await?;

        Ok(())
    }

    pub async fn remove_collaborator(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
        user_id: &Uuid,
    ) -> AppResult<()> {
        let quiz = self
            .policy
            .require_owner_quiz(current_user, quiz_id)
            .await?;

        if !self
            .repository
            .remove_collaborator(&quiz.id, user_id)
            .await?
        {
            return Err(QuizError::CollaboratorNotFound)?;
        }

        Ok(())
    }

    pub async fn list_collaborators(
        &self,
        current_user: &User,
        quiz_id: &Uuid,
    ) -> AppResult<Vec<User>> {
        let quiz = self
            .policy
            .require_managed_quiz(current_user, quiz_id)
            .await?;

        let mut users = self.repository.list_collaborator_users(&quiz.id).await?;

        if current_user.role == UserRole::Func && quiz.owner_id == current_user.id {
            return Ok(users);
        }

        users.retain(|user| user.id != quiz.owner_id);

        Ok(users)
    }

    pub fn generate_join_code_candidate() -> String {
        rand::rng()
            .sample_iter(Alphanumeric)
            .take(Self::JOIN_CODE_LENGTH)
            .map(char::from)
            .collect::<String>()
    }

    async fn generate_unique_join_code(&self) -> AppResult<String> {
        for _ in 0..Self::JOIN_CODE_MAX_ATTEMPTS {
            let join_code = Self::generate_join_code_candidate();

            if self.repository.find_by_code(&join_code).await?.is_none() {
                return Ok(join_code);
            }
        }

        Err(QuizError::InvalidCode.into())
    }

    async fn validate_collaborators(
        &self,
        collaborator_ids: &[Uuid],
        owner_id: &Uuid,
    ) -> AppResult<()> {
        for collaborator_id in collaborator_ids {
            if collaborator_id == owner_id {
                continue;
            }

            let Some(user) = self.users.find_by_id(collaborator_id).await? else {
                return Err(QuizError::CollaboratorNotFound)?;
            };

            if !matches!(user.role, UserRole::Assistant | UserRole::Func) {
                return Err(QuizError::InvalidCollaboratorRole)?;
            }
        }

        Ok(())
    }
}
