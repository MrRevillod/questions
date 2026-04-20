use crate::attempts::{AttemptError, AttemptService};
use crate::auth::SessionCheck;
use crate::authz::{AuthzAction, AuthzGuard};
use crate::quizzes::*;
use crate::users::User;

use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;
use uuid::Uuid;

#[controller(kind = Controller::Web, path = "/quizzes")]
#[interceptor(SessionCheck)]
pub struct QuizController {
    service: Arc<QuizService>,
    attempts: Arc<AttemptService>,
}

impl QuizController {
    #[get("/{quizId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::ReadManagedQuiz)]
    #[doc = "Get details of a quiz by its ID. Requires read access to the quiz (func|assistant)"]
    pub async fn get_detail(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let quiz = self.service.get_detail(&current_user, &quiz_id).await?;

        Ok(JsonResponse::Ok().data(quiz))
    }

    #[get("/me")]
    #[interceptor(AuthzGuard, config = AuthzAction::ListManagedQuizzes)]
    #[doc = "List all quizzes managed by the current user (func|assistant)"]
    pub async fn list_managed(&self, req: Request) -> WebResult {
        let current_user = req
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let quizzes = self.service.list_managed_by_user(&current_user).await?;

        Ok(JsonResponse::Ok().data(quizzes))
    }

    #[post("/")]
    #[interceptor(AuthzGuard, config = AuthzAction::CreateQuiz)]
    #[doc = "Create a new quiz. Requires create quiz permission (func|assistant)"]
    pub async fn create(&self, req: Request) -> WebResult {
        let current_user = req
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let input = req.body_validator::<CreateQuizRequest>()?;

        let quiz = self
            .service
            .create(&current_user, input, current_user.id)
            .await?;

        Ok(JsonResponse::Created().data(quiz))
    }

    #[patch("/{quizId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::UpdateManagedQuiz)]
    #[doc = "Update an existing quiz. Requires update access to the quiz (func|assistant)"]
    pub async fn update(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let input = req.body_validator::<UpdateQuizRequest>()?;
        let quiz = self.service.update(&current_user, &quiz_id, input).await?;

        Ok(JsonResponse::Ok().data(quiz))
    }

    #[delete("/{quizId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::DeleteManagedQuiz)]
    #[doc = "Delete an existing quiz. Only the quiz owner can perform this action."]
    pub async fn delete(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(JsonResponse::Unauthorized)?;

        self.service.delete_quiz(&current_user, &quiz_id).await?;

        Ok(JsonResponse::Ok().message("Quiz deleted successfully"))
    }

    #[post("/join-by-code")]
    #[interceptor(AuthzGuard, config = AuthzAction::JoinQuizByCode)]
    #[doc = "Join a quiz using a unique code. Requires join quiz permission (student)"]
    pub async fn join_by_code(&self, req: Request) -> WebResult {
        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let input = req.body_validator::<JoinQuizByCodeRequest>()?;

        let preview = self
            .service
            .get_join_preview(current_user, &input.code)
            .await?;

        Ok(JsonResponse::Ok().data(preview))
    }

    #[post("/{quizId}/attempts")]
    #[interceptor(AuthzGuard, config = AuthzAction::StartAttempt)]
    #[doc = "Start a new attempt for a quiz. Requires start attempt permission (student)"]
    pub async fn start_attempt(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let attempt = self.attempts.start_attempt(current_user, &quiz_id).await?;

        Ok(JsonResponse::Created().data(attempt))
    }

    #[get("/{quizId}/attempts")]
    #[interceptor(AuthzGuard, config = AuthzAction::ListManagedQuizAttempts)]
    pub async fn list_attempts(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let attempts = self
            .attempts
            .list_managed_quiz_attempts(current_user, &quiz_id)
            .await?;

        Ok(JsonResponse::Ok().data(attempts))
    }

    #[get("/{quizId}/attempts/{attemptId}/result")]
    #[interceptor(AuthzGuard, config = AuthzAction::ListManagedQuizAttempts)]
    pub async fn get_managed_attempt_result(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let attempt_id = req
            .param::<Uuid>("attemptId")
            .map_err(|_| AttemptError::InvalidAttemptId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let result = self
            .attempts
            .get_result_for_managed_attempt(current_user, &quiz_id, &attempt_id)
            .await?;

        Ok(JsonResponse::Ok().data(result))
    }

    #[get("/{quizId}/attempts/me")]
    #[interceptor(AuthzGuard, config = AuthzAction::StartAttempt)]
    #[doc = "Get the current active attempt for a quiz. Requires start attempt permission (student)"]
    pub async fn get_active_attempt(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let attempt = self
            .attempts
            .get_active_attempt_for_quiz(current_user, &quiz_id)
            .await?;

        Ok(JsonResponse::Ok().data(attempt))
    }

    #[get("/{quizId}/attempts/me/result")]
    #[interceptor(AuthzGuard, config = AuthzAction::ReadOwnAttemptResult)]
    pub async fn get_my_attempt_result(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let result = self
            .attempts
            .get_result_for_student(current_user, &quiz_id)
            .await?;

        Ok(JsonResponse::Ok().data(result))
    }

    #[post("/{quizId}/finalize-and-publish")]
    #[interceptor(AuthzGuard, config = AuthzAction::FinalizeManagedAttempt)]
    pub async fn finalize_and_publish(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let result = self
            .attempts
            .finalize_and_publish_quiz(current_user, &quiz_id)
            .await?;

        Ok(JsonResponse::Ok().data(result))
    }

    #[post("/results-by-code")]
    #[interceptor(AuthzGuard, config = AuthzAction::ReadOwnAttemptResult)]
    pub async fn get_my_attempt_result_by_code(&self, req: Request) -> WebResult {
        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let input = req.body_validator::<JoinQuizByCodeRequest>()?;
        let quiz_id = self
            .service
            .resolve_by_code_for_results(current_user, &input.code)
            .await?;

        let result = self
            .attempts
            .get_result_for_student(current_user, &quiz_id)
            .await?;

        Ok(JsonResponse::Ok().data(result))
    }

    #[put("/{quizId}/collaborators/{userId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::ManageQuizCollaborators)]
    #[doc = "Add a collaborator to a quiz. Requires manage collaborators permission (func|assistant)"]
    pub async fn add_collaborator(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let user_id = req
            .param::<Uuid>("userId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        self.service
            .add_collaborator(current_user, &quiz_id, AddCollaboratorRequest { user_id })
            .await?;

        Ok(JsonResponse::Ok().message("Collaborator added successfully"))
    }

    #[delete("/{quizId}/collaborators/{userId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::ManageQuizCollaborators)]
    #[doc = "Remove a collaborator from a quiz. Requires manage collaborators permission (func|assistant)"]
    pub async fn remove_collaborator(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let user_id = req
            .param::<Uuid>("userId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        self.service
            .remove_collaborator(current_user, &quiz_id, &user_id)
            .await?;

        Ok(JsonResponse::Ok().message("Collaborator removed successfully"))
    }

    #[get("/{quizId}/collaborators")]
    #[interceptor(AuthzGuard, config = AuthzAction::ReadManagedQuiz)]
    #[doc = "List all collaborators for a quiz. Requires read access to the quiz (func|assistant)"]
    pub async fn list_collaborators(&self, req: Request) -> WebResult {
        let quiz_id = req
            .param::<Uuid>("quizId")
            .map_err(|_| QuizError::InvalidId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let collaborators = self
            .service
            .list_collaborators(current_user, &quiz_id)
            .await?;

        Ok(JsonResponse::Ok().data(collaborators))
    }
}
