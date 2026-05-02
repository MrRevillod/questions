use std::sync::Arc;

use serde_json::json;
use sword::prelude::*;
use sword::web::*;
use uuid::Uuid;

use crate::attempts::*;
use crate::auth::SessionCheck;
use crate::authz::*;
use crate::courses::CourseId;
use crate::quizzes::QuizId;
use crate::shared::RequestExt;

#[controller(kind = Controller::Web, path = "/attempts")]
#[interceptor(SessionCheck)]
pub struct AttemptsController {
    attempts: Arc<AttemptsService>,
}

impl AttemptsController {
    #[get("/course/{courseId}/quiz/{quizId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::AttemptList)]
    async fn list(&self, req: Request) -> WebResult {
        let quiz_id = req.param::<QuizId>("quizId")?;
        let course_id = req.param::<CourseId>("courseId")?;

        let user = req.user().ok_or(JsonResponse::Unauthorized())?;

        let filter = AttemptFilter {
            course_id,
            quiz_id,
            student_id: None,
        };

        let attempts = self.attempts.list_attempts(user, filter).await?;

        Ok(JsonResponse::Ok().data(attempts))
    }

    #[post("/course/{courseId}/quiz/{quizId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::AttemptInitialize)]
    async fn initialize(&self, req: Request) -> WebResult {
        let quiz_id = req.param::<QuizId>("quizId")?;
        let user = req.user().ok_or(JsonResponse::Unauthorized())?;

        let attempt = self.attempts.initialize_attempt(quiz_id, user.id).await?;
        let (attempt, questions) = self.attempts.get_initialize_response(&attempt).await?;

        let data = json!({ "attempt": attempt, "questions": questions });

        Ok(JsonResponse::Created().data(data))
    }

    #[put("/{attemptId}/answers/{questionId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::AttemptSubmit)]
    async fn save_answer(&self, req: Request) -> WebResult {
        let attempt_id = req.param::<AttemptId>("attemptId")?;
        let question_id = req.param::<Uuid>("questionId")?;
        let input = req.body_validator::<SaveAttemptAnswerDto>()?;
        let user = req.user().ok_or(JsonResponse::Unauthorized())?;

        self.attempts
            .save_answer(attempt_id, question_id, user.id, input)
            .await?;

        Ok(JsonResponse::Ok())
    }

    #[post("/{attemptId}/submit")]
    #[interceptor(AuthzGuard, config = AuthzAction::AttemptSubmit)]
    async fn submit_attempt(&self, req: Request) -> WebResult {
        let attempt_id = req.param::<AttemptId>("attemptId")?;
        let user = req.user().ok_or(JsonResponse::Unauthorized())?;

        let attempt = self.attempts.submit_attempt(attempt_id, user.id).await?;

        let data = json!({ "attempt": attempt });

        Ok(JsonResponse::Ok().data(data))
    }

    #[get("/{attemptId}/result")]
    #[interceptor(AuthzGuard, config = AuthzAction::AttemptViewResults)]
    async fn view_result(&self, req: Request) -> WebResult {
        let attempt_id = req.param::<AttemptId>("attemptId")?;
        let user = req.user().ok_or(JsonResponse::Unauthorized())?;

        let result = self.attempts.view_results(attempt_id, user.id).await?;

        let data = json!(result);

        Ok(JsonResponse::Ok().data(data))
    }

    #[get("/{attemptId}/results")]
    #[interceptor(AuthzGuard, config = AuthzAction::AttemptViewResults)]
    async fn view_results_legacy(&self, req: Request) -> WebResult {
        let attempt_id = req.param::<AttemptId>("attemptId")?;
        let user = req.user().ok_or(JsonResponse::Unauthorized())?;

        let result = self.attempts.view_results(attempt_id, user.id).await?;

        let data = json!(result);

        Ok(JsonResponse::Ok().data(data))
    }
}
