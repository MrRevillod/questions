use crate::auth::SessionCheck;
use crate::authz::{AuthzAction, AuthzGuard};
use crate::quizzes::*;
use crate::shared::RequestExt;

use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/quizzes")]
#[interceptor(SessionCheck)]
pub struct QuizController {
    service: Arc<QuizService>,
}

impl QuizController {
    #[get("/{quizId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::ReadManagedQuiz)]
    #[doc = "Get details of a quiz by its ID. Requires read access to the quiz (func|assistant)"]
    pub async fn get_detail(&self, req: Request) -> WebResult {
        let quiz_id = req.param::<QuizId>("quizId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        let quiz = self.service.get_one(current_user, &quiz_id).await?;

        Ok(JsonResponse::Ok().data(quiz))
    }

    #[get("/me")]
    #[interceptor(AuthzGuard, config = AuthzAction::ListManagedQuizzes)]
    #[doc = "List all quizzes managed by the current user (func|assistant)"]
    pub async fn list_managed(&self, req: Request) -> WebResult {
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;
        let quizzes = self.service.list_managed_by_user(current_user).await?;

        Ok(JsonResponse::Ok().data(quizzes))
    }

    #[post("/")]
    #[interceptor(AuthzGuard, config = AuthzAction::CreateQuiz)]
    #[doc = "Create a new quiz. Requires create quiz permission (func|assistant)"]
    pub async fn create(&self, req: Request) -> WebResult {
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;
        let input = req.body_validator::<CreateQuizDto>()?;

        let quiz = self.service.create(current_user, input).await?;

        Ok(JsonResponse::Created().data(quiz))
    }

    #[delete("/{quizId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::DeleteManagedQuiz)]
    #[doc = "Delete an existing quiz. Only the quiz owner can perform this action."]
    pub async fn delete(&self, req: Request) -> WebResult {
        let quiz_id = req.param::<QuizId>("quizId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        self.service.delete_quiz(current_user, &quiz_id).await?;

        Ok(JsonResponse::Ok().message("Quiz deleted successfully"))
    }

    #[post("/join/{joinCode}")]
    #[interceptor(AuthzGuard, config = AuthzAction::JoinQuizByCode)]
    #[doc = "Join a quiz using a unique code. Requires join quiz permission (student)"]
    pub async fn join_by_code(&self, req: Request) -> WebResult {
        let code = req.param::<String>("joinCode")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        let preview = self.service.get_join_preview(current_user, &code).await?;

        Ok(JsonResponse::Ok().data(preview))
    }
}
