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
    #[interceptor(AuthzGuard, config = AuthzAction::QuizReadManaged)] // (func | assistant)
    pub async fn get_one(&self, req: Request) -> WebResult {
        let quiz_id = req.param::<QuizId>("quizId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        let quiz = self.service.get_one(current_user, &quiz_id).await?;

        Ok(JsonResponse::Ok().data(quiz))
    }

    #[get("/me")]
    #[interceptor(AuthzGuard, config = AuthzAction::QuizListManaged)] // (func | assistant)
    pub async fn list_managed(&self, req: Request) -> WebResult {
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;
        let quizzes = self.service.list_managed_by_user(current_user).await?;

        Ok(JsonResponse::Ok().data(quizzes))
    }

    #[post("/")]
    #[interceptor(AuthzGuard, config = AuthzAction::QuizCreate)] // (func | assistant)
    pub async fn create(&self, req: Request) -> WebResult {
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;
        let input = req.body_validator::<CreateQuizDto>()?;

        let quiz = self.service.create(current_user, input).await?;

        Ok(JsonResponse::Created().data(quiz))
    }

    #[delete("/{quizId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::QuizDeleteManaged)] // (func | assistant)
    pub async fn delete(&self, req: Request) -> WebResult {
        let quiz_id = req.param::<QuizId>("quizId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        self.service.delete_quiz(current_user, &quiz_id).await?;

        Ok(JsonResponse::Ok().message("Quiz deleted successfully"))
    }

    #[post("/join/{joinCode}")]
    #[interceptor(AuthzGuard, config = AuthzAction::QuizJoinByCode)] // (func | assistant)
    pub async fn join_by_code(&self, req: Request) -> WebResult {
        let code = req.param::<String>("joinCode")?;
        let preview = self.service.get_join_preview(&code).await?;

        Ok(JsonResponse::Ok().data(preview))
    }

    #[get("/join/{joinCode}/attempts/me/result")]
    #[interceptor(AuthzGuard, config = AuthzAction::QuizViewAttemptResultByCode)]
    pub async fn get_my_result_by_code(&self, req: Request) -> WebResult {
        let code = req.param::<String>("joinCode")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        let result = self
            .service
            .get_my_result_by_join_code(current_user, &code)
            .await?;

        Ok(JsonResponse::Ok().data(result))
    }
}
