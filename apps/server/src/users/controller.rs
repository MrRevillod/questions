use crate::auth::SessionCheck;
use crate::authz::{AuthzAction, AuthzGuard};
use crate::shared::RequestExt;
use crate::users::*;

use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/users")]
#[interceptor(SessionCheck)]
pub struct UsersController {
    users: Arc<UserRepository>,
    service: Arc<UsersService>,
}

impl UsersController {
    #[get("/me")]
    #[doc = "Get the current authenticated user's information"]
    pub async fn get_me(&self, req: Request) -> WebResult {
        let claims = req.claims().ok_or_else(JsonResponse::Unauthorized)?;

        let user = self
            .users
            .find_by_id(&claims.user_id)
            .await?
            .ok_or_else(|| UsersError::NotFound(claims.user_id.to_string()))?;

        Ok(JsonResponse::Ok().data(user))
    }

    #[get("/")]
    #[interceptor(AuthzGuard, config = AuthzAction::ListUsersAdmin)]
    #[doc = "List all users in system (admin only) with all details."]
    pub async fn list_users(&self, req: Request) -> WebResult {
        let query = req.query::<SearchUsersQuery>()?.unwrap_or_default();
        let users = self.service.list_users(query).await?;

        Ok(JsonResponse::Ok().data(users))
    }

    #[get("/collaborator-candidates")]
    #[interceptor(AuthzGuard, config = AuthzAction::ListCollaboratorCandidates)]
    #[doc = "List users (teachers and assistants) who can be added as test collaborators"]
    pub async fn list_collaborator_candidates(&self, req: Request) -> WebResult {
        let query = req.query::<SearchUsersQuery>()?.unwrap_or_default();
        let users = self.service.list_collaborator_candidates(query).await?;

        Ok(JsonResponse::Ok().data(users))
    }

    #[patch("/{userId}/role")]
    #[interceptor(AuthzGuard, config = AuthzAction::ManageAssistants)]
    #[doc = "Update a student role to 'assistant' (admin executable only)"]
    pub async fn set_user_role(&self, req: Request) -> WebResult {
        let user_id = req.param::<UserId>("userId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        let input = req.body_validator::<UpdateUserRoleRequest>()?;

        let updated_user = self
            .service
            .update_role(current_user, &user_id, input)
            .await?;

        Ok(JsonResponse::Ok().data(updated_user))
    }
}
