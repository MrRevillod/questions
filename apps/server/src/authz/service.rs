use crate::{authz::AuthzAction, authz::AuthzError, shared::AppResult, users::UserRole};

use sword::prelude::*;

#[injectable]
pub struct AuthzService;

impl AuthzService {
    pub fn authorize_role(&self, role: &UserRole, action: AuthzAction) -> AppResult<()> {
        let allowed = match role {
            UserRole::Func => matches!(
                action,
                AuthzAction::CreateQuiz
                    | AuthzAction::ListManagedQuizzes
                    | AuthzAction::ReadManagedQuiz
                    | AuthzAction::UpdateManagedQuiz
                    | AuthzAction::ManageQuizCollaborators
                    | AuthzAction::JoinQuizByCode
                    | AuthzAction::StartAttempt
                    | AuthzAction::ListManagedQuizAttempts
                    | AuthzAction::SaveOwnAttemptAnswer
                    | AuthzAction::SubmitOwnAttempt
                    | AuthzAction::ReadOwnAttemptResult
                    | AuthzAction::FinalizeManagedAttempt
                    | AuthzAction::ListUsersAdmin
                    | AuthzAction::ListCollaboratorCandidates
                    | AuthzAction::ManageAssistants
            ),
            UserRole::Assistant => matches!(
                action,
                AuthzAction::CreateQuiz
                    | AuthzAction::ListManagedQuizzes
                    | AuthzAction::ReadManagedQuiz
                    | AuthzAction::UpdateManagedQuiz
                    | AuthzAction::ListManagedQuizAttempts
                    | AuthzAction::ListCollaboratorCandidates
                    | AuthzAction::ManageQuizCollaborators
                    | AuthzAction::FinalizeManagedAttempt
            ),
            UserRole::Student => matches!(
                action,
                AuthzAction::JoinQuizByCode
                    | AuthzAction::StartAttempt
                    | AuthzAction::SaveOwnAttemptAnswer
                    | AuthzAction::SubmitOwnAttempt
                    | AuthzAction::ReadOwnAttemptResult
            ),
        };

        if !allowed {
            tracing::warn!(role = ?role, action = ?action, "AuthzService denied action for role");
            return Err(AuthzError::Forbidden(action))?;
        }

        tracing::debug!(role = ?role, action = ?action, "AuthzService allowed action for role");

        Ok(())
    }
}
