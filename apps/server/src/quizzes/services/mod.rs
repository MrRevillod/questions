mod codegen;
mod policy;

pub use codegen::*;
pub use policy::*;

use std::str::FromStr;
use std::sync::Arc;

use crate::quizzes::QuizQuestion;
use crate::quizzes::*;
use crate::shared::AppResult;
use crate::users::{User, UserId, UserRepository, UserRole};

use chrono::{DateTime, Utc};
use sword::prelude::*;

#[injectable]
pub struct QuizService {
    codegen: Arc<QuizCodeGenerator>,
    policy: Arc<QuizPolicy>,
    repository: Arc<QuizRepository>,
    users: Arc<UserRepository>,
}

impl QuizService {
    pub async fn get_detail(
        &self,
        current_user: &User,
        quiz_id: &QuizId,
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
            return Err(QuizError::Closed)?;
        }

        Ok(JoinQuizPreviewView::from(&quiz))
    }

    pub async fn resolve_by_code_for_results(
        &self,
        current_user: &User,
        code: &str,
    ) -> AppResult<QuizId> {
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
        owner_id: UserId,
    ) -> AppResult<QuizDetailView> {
        self.policy.can_create_quiz(current_user)?;

        self.validate_collaborators(&input.collaborator_ids, &owner_id)
            .await?;

        let join_code = self.codegen.generate_unique_join_code().await?;
        let now = Utc::now();

        let quiz = Quiz::builder()
            .owner_id(owner_id)
            .title(input.title)
            .kind(QuizKind::from_str(&input.mode)?)
            .join_code(join_code)
            .start_time(
                DateTime::parse_from_rfc3339(&input.start_time_utc)
                    .map_err(|_| QuizError::InvalidStartTime)?
                    .with_timezone(&Utc),
            )
            .created_at(now)
            .updated_at(now)
            .questions(input.questions.iter().map(QuizQuestion::from).collect())
            .attempt_duration_minutes(input.attempt_duration_minutes)
            .question_count(input.question_count)
            .maybe_certainly_table(input.certainty_config.map(CertainlyTable::from))
            .build();

        let quiz = self.repository.create(quiz).await?;

        for collaborator_id in input.collaborator_ids {
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
        quiz_id: &QuizId,
        input: UpdateQuizRequest,
    ) -> AppResult<QuizDetailView> {
        let mut quiz = self
            .policy
            .require_managed_quiz(current_user, quiz_id)
            .await?;

        self.policy.can_update_quiz(current_user, &quiz).await?;

        if self.repository.has_attempts(quiz_id).await? {
            return Err(QuizError::LockedForAttempts.into());
        }

        if let Some(start_time_utc) = input.start_time_utc {
            quiz.start_time = DateTime::parse_from_rfc3339(&start_time_utc)
                .map_err(|_| QuizError::InvalidStartTime)?
                .with_timezone(&Utc);
        }

        if let Some(title) = input.title {
            quiz.title = title;
        }

        if let Some(attempt_duration_minutes) = input.attempt_duration_minutes {
            quiz.attempt_duration_minutes = attempt_duration_minutes;
        }

        if let Some(question_count) = input.question_count {
            quiz.question_count = question_count;
        }

        if let Some(certainly_config) = input.certainty_config {
            quiz.certainly_table = Some(CertainlyTable::from(certainly_config));
        }

        if let Some(questions) = input.questions {
            quiz.questions = questions.iter().map(QuizQuestion::from).collect();
        }

        if quiz.question_count as usize > quiz.questions.len() {
            return Err(QuizError::InvalidQuestionCount.into());
        }

        let quiz = self.repository.update(&quiz).await?;

        Ok(QuizDetailView::from(quiz))
    }

    pub async fn delete_quiz(&self, current_user: &User, quiz_id: &QuizId) -> AppResult<()> {
        let quiz = self
            .policy
            .require_owner_quiz(current_user, quiz_id)
            .await?;

        if !self.repository.delete_by_id(&quiz.id).await? {
            return Err(QuizError::NotFound(quiz_id.to_string()))?;
        }

        Ok(())
    }

    pub async fn add_collaborator(
        &self,
        current_user: &User,
        quiz_id: &QuizId,
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
        quiz_id: &QuizId,
        user_id: &UserId,
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
        quiz_id: &QuizId,
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

    async fn validate_collaborators(
        &self,
        collaborator_ids: &[UserId],
        owner_id: &UserId,
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
