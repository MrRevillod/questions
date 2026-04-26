mod ldap;

use crate::{
    auth::*,
    shared::*,
    users::{User, UserId, UserRepository},
};

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use sword::prelude::*;

pub use ldap::LdapClient;

#[injectable]
pub struct AuthService {
    config: AuthConfig,
    ldap: Arc<LdapClient>,
    users: Arc<UserRepository>,
    jwt_service: Arc<JsonWebTokenService>,
    sessions: Arc<SessionRepository>,
}

impl AuthService {
    pub async fn login(&self, input: LoginDto) -> AppResult<LoginResponse> {
        let LoginDto { username, password } = input;

        let ldap_user = self.ldap.authenticate(&username, &password).await?;

        let user = match self.users.find_by_username(&username).await? {
            Some(existing) => existing,
            None => {
                let incoming_user = User::builder()
                    .username(username)
                    .email(ldap_user.email)
                    .name(ldap_user.name)
                    .role(ldap_user.role)
                    .build();

                self.users.save(&incoming_user).await?
            }
        };

        let session_id = SessionId::new();
        let access_token = self.generate_access_token(&session_id, &user.id)?;
        let refresh_token = self.generate_refresh_token(&session_id, &user.id)?;
        let now = Utc::now();

        let session = Session {
            id: session_id,
            user_id: user.id,
            refresh_token_hash: Self::hash_token(&refresh_token),
            created_at: now,
            expires_at: now + Duration::days(self.config.session_exp_days),
            refresh_expires_at: now + Duration::days(self.config.refresh_exp_days),
            revoked_at: None,
        };

        self.sessions.save(&session).await?;

        Ok(LoginResponse {
            user,
            access_token,
            refresh_token,
        })
    }

    pub async fn refresh(&self, token: &str) -> AppResult<RefreshResponse> {
        let claims: SessionClaims = self
            .jwt_service
            .decode(&token.to_string(), self.config.jwt_secret.as_ref())?;

        if claims.typ != "refresh" {
            return Err(AppError::InvalidToken);
        }

        let mut session = self
            .sessions
            .find_active_by_id(&claims.session_id)
            .await?
            .ok_or(AppError::TokenNotFound)?;

        if session.refresh_expires_at <= Utc::now() {
            session.revoked_at = Some(Utc::now());
            self.sessions.save(&session).await?;

            return Err(AppError::TokenNotFound);
        }

        let incoming_refresh_hash = Self::hash_token(token);

        if incoming_refresh_hash != session.refresh_token_hash {
            session.revoked_at = Some(Utc::now());
            self.sessions.save(&session).await?;

            return Err(AppError::InvalidToken);
        }

        let access_token = self.generate_access_token(&session.id, &session.user_id)?;
        let refresh_token = self.generate_refresh_token(&session.id, &session.user_id)?;

        session.refresh_token_hash = Self::hash_token(&refresh_token);
        session.refresh_expires_at = Utc::now() + Duration::days(self.config.refresh_exp_days);

        self.sessions.save(&session).await?;

        Ok(RefreshResponse {
            access_token,
            refresh_token,
        })
    }

    pub async fn logout(&self, session_id: &SessionId) -> AppResult<()> {
        if let Some(mut session) = self.sessions.find_active_by_id(session_id).await? {
            session.revoked_at = Some(Utc::now());
            self.sessions.save(&session).await?;
        }

        Ok(())
    }

    fn generate_access_token(&self, session_id: &SessionId, user_id: &UserId) -> AppResult<String> {
        let claims = SessionClaims {
            session_id: *session_id,
            user_id: *user_id,
            exp: (Utc::now() + Duration::minutes(self.config.access_exp_minutes)).timestamp(),
            typ: "access".to_string(),
        };

        let token = self
            .jwt_service
            .encode(&claims, self.config.jwt_secret.as_ref())?;

        Ok(token)
    }

    fn generate_refresh_token(
        &self,
        session_id: &SessionId,
        user_id: &UserId,
    ) -> AppResult<String> {
        let claims = SessionClaims {
            session_id: *session_id,
            user_id: *user_id,
            exp: (Utc::now() + Duration::days(self.config.refresh_exp_days)).timestamp(),
            typ: "refresh".to_string(),
        };

        let token = self
            .jwt_service
            .encode(&claims, self.config.jwt_secret.as_ref())?;

        Ok(token)
    }

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
