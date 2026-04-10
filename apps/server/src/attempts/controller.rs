use crate::attempts::*;
use crate::auth::SessionCheck;
use crate::authz::{AuthzAction, AuthzGuard};
use crate::users::User;

use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;
use uuid::Uuid;

#[controller(kind = Controller::Web, path = "/attempts")]
#[interceptor(SessionCheck)]
pub struct AttemptController {
    service: Arc<AttemptService>,
}

impl AttemptController {
    #[put("/{attemptId}/answers/{questionId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::SaveOwnAttemptAnswer)]
    #[doc = "Save or update the answer for a specific question in an attempt."]
    #[doc = "Requires permission to save answers for own attempts (func|assistant)."]
    pub async fn save_answer(&self, req: Request) -> WebResult {
        let attempt_id = req
            .param::<Uuid>("attemptId")
            .map_err(|_| AttemptError::InvalidAttemptId)?;

        let question_id = req
            .param::<Uuid>("questionId")
            .map_err(|_| AttemptError::InvalidQuestionId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let input = req.body_validator::<SaveAnswerRequest>()?;

        let input = SaveAnswerCommand {
            attempt_id,
            question_id,
            answer_index: input.answer_index,
            certainty_level: input.certainty_level,
        };

        let answer = self.service.save_answer(&current_user, input).await?;

        Ok(JsonResponse::Ok().data(answer))
    }

    #[post("/{attemptId}/submit")]
    #[interceptor(AuthzGuard, config = AuthzAction::SubmitOwnAttempt)]
    pub async fn submit(&self, req: Request) -> WebResult {
        let attempt_id = req
            .param::<Uuid>("attemptId")
            .map_err(|_| AttemptError::InvalidAttemptId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let attempt = self
            .service
            .submit_attempt(&current_user, &attempt_id)
            .await?;

        Ok(JsonResponse::Ok().data(attempt))
    }

    #[get("/{attemptId}/result")]
    #[interceptor(AuthzGuard, config = AuthzAction::ReadOwnAttemptResult)]
    pub async fn get_result(&self, req: Request) -> WebResult {
        let attempt_id = req
            .param::<Uuid>("attemptId")
            .map_err(|_| AttemptError::InvalidAttemptId)?;

        let current_user = req
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(JsonResponse::Unauthorized)?;

        let result = self
            .service
            .get_result_for_attempt_owner(&current_user, &attempt_id)
            .await?;

        Ok(JsonResponse::Ok().data(result))
    }
}
