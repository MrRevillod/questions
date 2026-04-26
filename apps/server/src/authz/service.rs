use crate::{authz::AuthzAction, authz::AuthzError, shared::AppResult, users::UserRole};

use sword::prelude::*;

#[injectable]
pub struct AuthzService;

impl AuthzService {
    pub fn authorize_role(&self, role: &UserRole, action: AuthzAction) -> AppResult<()> {
        let allowed = match role {
            UserRole::Func => matches!(
                action,
                AuthzAction::CreateCourse
                    | AuthzAction::ListCourses
                    | AuthzAction::ReadCourse
                    | AuthzAction::DeleteCourse
                    | AuthzAction::ManageCourseMembers
                    | AuthzAction::CreateQuestionBank
                    | AuthzAction::ListQuestionBanks
                    | AuthzAction::ReadQuestionBank
                    | AuthzAction::UpdateQuestionBank
                    | AuthzAction::DeleteQuestionBank
                    | AuthzAction::CreateQuiz
                    | AuthzAction::ListManagedQuizzes
                    | AuthzAction::ReadManagedQuiz
                    | AuthzAction::UpdateManagedQuiz
                    | AuthzAction::ManageQuizCollaborators
                    | AuthzAction::JoinQuizByCode
                    | AuthzAction::DeleteManagedQuiz
                    | AuthzAction::ListUsersAdmin
                    | AuthzAction::ListCollaboratorCandidates
                    | AuthzAction::ManageAssistants
            ),
            UserRole::Assistant => matches!(
                action,
                AuthzAction::CreateCourse
                    | AuthzAction::ListCourses
                    | AuthzAction::ReadCourse
                    | AuthzAction::CreateQuestionBank
                    | AuthzAction::ListQuestionBanks
                    | AuthzAction::ReadQuestionBank
                    | AuthzAction::UpdateQuestionBank
                    | AuthzAction::DeleteQuestionBank
                    | AuthzAction::CreateQuiz
                    | AuthzAction::ListManagedQuizzes
                    | AuthzAction::ReadManagedQuiz
                    | AuthzAction::UpdateManagedQuiz
                    | AuthzAction::ListCollaboratorCandidates
                    | AuthzAction::ManageQuizCollaborators
                    | AuthzAction::DeleteManagedQuiz
            ),
            UserRole::Student => matches!(action, AuthzAction::JoinQuizByCode),
            UserRole::Admin => true,
        };

        if !allowed {
            tracing::warn!(role = ?role, action = ?action, "AuthzService denied action for role");
            return Err(AuthzError::Forbidden(action))?;
        }

        tracing::debug!(role = ?role, action = ?action, "AuthzService allowed action for role");

        Ok(())
    }
}
