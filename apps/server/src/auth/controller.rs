use crate::auth::*;
use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/auth")]
pub struct AuthController {
    auth_service: Arc<AuthService>,
}

impl AuthController {
    #[post("/login")]
    pub async fn login(&self, req: Request) -> WebResult {
        let dto = req.body::<LoginDto>()?;

        tracing::info!("User login attempt for username: {}", dto.username);

        let response = self.auth_service.login(dto).await?;

        Ok(JsonResponse::Ok().data(response))
    }

    #[post("/refresh")]
    pub async fn refresh(&self, req: Request) -> WebResult {
        let auth_header = req.authorization().ok_or_else(JsonResponse::Unauthorized)?;

        let Some(token) = auth_header.strip_prefix("Bearer ") else {
            return Err(JsonResponse::Unauthorized());
        };

        let response = self.auth_service.refresh(&token).await?;

        Ok(JsonResponse::Ok().data(response))
    }

    #[post("/logout")]
    #[interceptor(SessionCheck)]
    pub async fn logout(&self, req: Request) -> WebResult {
        let session_claims = req
            .extensions
            .get::<SessionClaims>()
            .ok_or_else(JsonResponse::Unauthorized)?;

        self.auth_service.logout(&session_claims.session_id).await?;

        Ok(JsonResponse::Ok().message("Logged out successfully"))
    }
}
