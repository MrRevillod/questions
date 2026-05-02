use crate::{
    auth::SessionClaims,
    authz::{AuthzAction, AuthzError, AuthzService},
    users::{User, UserRepository},
};

use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;

#[derive(Interceptor)]
pub struct AuthzGuard {
    authz: Arc<AuthzService>,
    users: Arc<UserRepository>,
}

impl OnRequestWithConfig<AuthzAction> for AuthzGuard {
    async fn on_request(&self, action: AuthzAction, mut req: Request) -> WebInterceptorResult {
        let method = req.method().to_string();
        let path = req.uri();

        let claims = req
            .extensions
            .get::<SessionClaims>()
            .cloned()
            .ok_or_else(|| {
                tracing::warn!(
                    method = %method,
                    path = %path,
                    action = ?action,
                    "AuthzGuard rejected: session claims missing"
                );
                JsonResponse::Unauthorized()
            })?;

        let Some(user) = self.users.find_by_id(&claims.user_id).await? else {
            tracing::warn!(
                method = %method,
                path = %path,
                action = ?action,
                user_id = %claims.user_id,
                session_id = %claims.session_id,
                "AuthzGuard rejected: actor not found"
            );

            return Err(AuthzError::ActorNotFound(claims.user_id.to_string()))?;
        };

        self.authz.authorize_role(&user.role, action)?;

        tracing::debug!(
            method = %method,
            path = %path,
            action = ?action,
            user_id = %user.id,
            username = %user.username,
            role = ?user.role,
            "AuthzGuard accepted"
        );

        req.extensions.insert::<User>(user);

        req.next().await
    }
}
