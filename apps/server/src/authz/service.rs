use crate::{
    authz::{AuthzAction, AuthzError},
    shared::AppResult,
    users::UserRole,
};

use sword::prelude::*;

#[injectable]
pub struct AuthzService;

impl AuthzService {
    pub fn authorize_role(&self, role: &UserRole, action: AuthzAction) -> AppResult<()> {
        let allowed = match role {
            UserRole::Admin => true,
            UserRole::Func => self.can_func(action),
            UserRole::Assistant => self.can_assistant(action),
            UserRole::Student => self.can_student(action),
        };

        if !allowed {
            tracing::warn!(role = ?role, action = ?action, "AuthzService denied action for role");
            return Err(AuthzError::Forbidden(action))?;
        }

        tracing::debug!(role = ?role, action = ?action, "AuthzService allowed action for role");

        Ok(())
    }

    fn can_func(&self, action: AuthzAction) -> bool {
        is_course_action(action)
            || is_bank_action(action)
            || is_quiz_management_action(action)
            || is_student_quiz_action(action)
            || is_attempt_action(action)
            || is_user_management_action(action)
    }

    fn can_assistant(&self, action: AuthzAction) -> bool {
        is_assistant_course_action(action)
            || is_bank_action(action)
            || is_quiz_management_action(action)
            || is_attempt_action(action)
            || is_assistant_user_action(action)
    }

    fn can_student(&self, action: AuthzAction) -> bool {
        is_student_quiz_action(action) || is_attempt_action(action)
    }
}

fn is_bank_action(action: AuthzAction) -> bool {
    matches!(
        action,
        AuthzAction::BankList
            | AuthzAction::BankRead
            | AuthzAction::BankCreate
            | AuthzAction::BankUpdate
            | AuthzAction::BankDelete
    )
}

fn is_user_management_action(action: AuthzAction) -> bool {
    matches!(
        action,
        AuthzAction::UserListAdmin
            | AuthzAction::UserListCollaboratorCandidates
            | AuthzAction::UserManageAssistants
    )
}

fn is_assistant_user_action(action: AuthzAction) -> bool {
    matches!(action, AuthzAction::UserListCollaboratorCandidates)
}

fn is_quiz_management_action(action: AuthzAction) -> bool {
    matches!(
        action,
        AuthzAction::QuizReadManaged
            | AuthzAction::QuizListManaged
            | AuthzAction::QuizCreate
            | AuthzAction::QuizUpdateManaged
            | AuthzAction::QuizManageCollaborators
            | AuthzAction::QuizDeleteManaged
    )
}

fn is_student_quiz_action(action: AuthzAction) -> bool {
    matches!(
        action,
        AuthzAction::QuizJoinByCode | AuthzAction::QuizViewAttemptResultByCode
    )
}

fn is_attempt_action(action: AuthzAction) -> bool {
    matches!(
        action,
        AuthzAction::AttemptList
            | AuthzAction::AttemptInitialize
            | AuthzAction::AttemptSubmit
            | AuthzAction::AttemptViewResults
    )
}

fn is_course_action(action: AuthzAction) -> bool {
    matches!(
        action,
        AuthzAction::CourseList
            | AuthzAction::CourseRead
            | AuthzAction::CourseCreate
            | AuthzAction::CourseDelete
            | AuthzAction::CourseManageMembers
    )
}

fn is_assistant_course_action(action: AuthzAction) -> bool {
    matches!(
        action,
        AuthzAction::CourseList | AuthzAction::CourseRead | AuthzAction::CourseCreate
    )
}
