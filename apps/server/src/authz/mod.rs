mod errors;
mod interceptor;
mod service;

use sword::prelude::*;

pub use errors::AuthzError;
pub use interceptor::AuthzGuard;
pub use service::AuthzService;

#[derive(Clone, Copy, Debug)]
pub enum AuthzAction {
    CreateCourse,
    ListCourses,
    ReadCourse,
    DeleteCourse,
    ManageCourseMembers,
    CreateQuestionBank,
    ListQuestionBanks,
    ReadQuestionBank,
    UpdateQuestionBank,
    DeleteQuestionBank,
    CreateQuiz,
    ListManagedQuizzes,
    ReadManagedQuiz,
    UpdateManagedQuiz,
    ManageQuizCollaborators,
    JoinQuizByCode,
    DeleteManagedQuiz,
    ListUsersAdmin,
    ListCollaboratorCandidates,
    ManageAssistants,
}

pub struct AuthzModule;

impl Module for AuthzModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<AuthzService>();
    }
}
