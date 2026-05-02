mod errors;
mod interceptor;
mod service;

use sword::prelude::*;

pub use errors::AuthzError;
pub use interceptor::AuthzGuard;
pub use service::AuthzService;

#[derive(Clone, Copy, Debug)]
pub enum AuthzAction {
    CourseList,
    CourseRead,
    CourseCreate,
    CourseDelete,
    CourseManageMembers,

    BankList,
    BankRead,
    BankCreate,
    BankUpdate,
    BankDelete,

    QuizReadManaged,
    QuizListManaged,
    QuizCreate,
    QuizUpdateManaged,
    QuizManageCollaborators,
    QuizJoinByCode,
    QuizViewAttemptResultByCode,
    QuizDeleteManaged,

    AttemptList,
    AttemptInitialize,
    AttemptSubmit,
    AttemptViewResults,

    UserListAdmin,
    UserListCollaboratorCandidates,
    UserManageAssistants,
}

pub struct AuthzModule;

impl Module for AuthzModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<AuthzService>();
    }
}
