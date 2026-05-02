use crate::auth::SessionCheck;
use crate::authz::{AuthzAction, AuthzGuard};
use crate::banks::*;
use crate::courses::CourseId;
use crate::shared::RequestExt;

use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/banks")]
#[interceptor(SessionCheck)]
pub struct QuestionBankController {
    service: Arc<QuestionBankService>,
}

impl QuestionBankController {
    #[get("/course/{courseId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::BankList)] // (func | assistant)
    pub async fn list_banks(&self, req: Request) -> WebResult {
        let course_id = req.param::<CourseId>("courseId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        let banks = self
            .service
            .list_for_course(current_user, &course_id)
            .await?;

        Ok(JsonResponse::Ok().data(banks))
    }

    #[get("/{bankId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::BankRead)] // (func | assistant)
    pub async fn get_bank(&self, req: Request) -> WebResult {
        let bank_id = req.param::<QuestionBankId>("bankId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        let bank = self.service.get_one(current_user, &bank_id).await?;

        Ok(JsonResponse::Ok().data(bank))
    }

    #[post("/")]
    #[interceptor(AuthzGuard, config = AuthzAction::BankCreate)]
    pub async fn create_bank(&self, req: Request) -> WebResult {
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;
        let input = req.body_validator::<CreateQuestionBankDto>()?;

        self.service.create(current_user, input).await?;

        Ok(JsonResponse::Created().message("Question bank created successfully"))
    }

    #[patch("/{bankId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::BankUpdate)]
    pub async fn update_bank(&self, req: Request) -> WebResult {
        let bank_id = req.param::<QuestionBankId>("bankId")?;
        let input = req.body_validator::<UpdateQuestionBankDto>()?;

        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        self.service.update(current_user, &bank_id, input).await?;

        Ok(JsonResponse::Ok().message("Question bank updated successfully"))
    }

    #[delete("/{bankId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::BankDelete)]
    pub async fn delete_bank(&self, req: Request) -> WebResult {
        let bank_id = req.param::<QuestionBankId>("bankId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        self.service.soft_delete(current_user, &bank_id).await?;

        Ok(JsonResponse::Ok().message("Question bank deleted successfully"))
    }
}
