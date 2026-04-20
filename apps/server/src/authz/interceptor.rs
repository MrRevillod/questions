use crate::{
    auth::{AuthConfig, SessionClaims, SessionRepository},
    authz::{AuthzAction, AuthzError, AuthzService},
    shared::JsonWebTokenService,
    users::{User, UserRepository},
};

use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;

#[derive(Interceptor)]
pub struct AuthzGuard {
    config: AuthConfig,
    jwt_service: Arc<JsonWebTokenService>,
    sessions: Arc<SessionRepository>,
    authz: Arc<AuthzService>,
    users: Arc<UserRepository>,
}

impl OnRequestWithConfig<AuthzAction> for AuthzGuard {
    async fn on_request(&self, action: AuthzAction, mut req: Request) -> WebInterceptorResult {
        let method = req.method().to_string();
        let path = req.uri();

        let claims = match req.extensions.get::<SessionClaims>().cloned() {
            Some(claims) => claims,
            None => {
                tracing::warn!(
                    method = %method,
                    path = %path,
                    action = ?action,
                    "AuthzGuard: session claims missing, attempting inline token validation"
                );

                let auth_header = req.authorization().ok_or_else(|| {
                    tracing::warn!(
                        method = %method,
                        path = %path,
                        action = ?action,
                        "AuthzGuard rejected: missing Authorization header"
                    );
                    JsonResponse::Unauthorized()
                })?;

                let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
                    tracing::warn!(
                        method = %method,
                        path = %path,
                        action = ?action,
                        "AuthzGuard rejected: invalid Authorization scheme"
                    );
                    JsonResponse::Unauthorized()
                })?;

                let claims: SessionClaims = self
                    .jwt_service
                    .decode(&token.to_string(), self.config.jwt_secret.as_ref())
                    .map_err(|error| {
                        tracing::warn!(
                            method = %method,
                            path = %path,
                            action = ?action,
                            error = %error,
                            "AuthzGuard rejected: token decode failed"
                        );
                        JsonResponse::Unauthorized()
                    })?;

                if claims.typ != "access" {
                    tracing::warn!(
                        method = %method,
                        path = %path,
                        action = ?action,
                        token_type = %claims.typ,
                        "AuthzGuard rejected: token type is not access"
                    );
                    return Err(JsonResponse::Unauthorized());
                }

                if !self.sessions.is_active(&claims.session_id).await? {
                    tracing::warn!(
                        method = %method,
                        path = %path,
                        action = ?action,
                        session_id = %claims.session_id,
                        user_id = %claims.user_id,
                        "AuthzGuard rejected: session is not active"
                    );
                    return Err(JsonResponse::Unauthorized());
                }

                req.extensions.insert::<SessionClaims>(claims.clone());
                claims
            }
        };

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
