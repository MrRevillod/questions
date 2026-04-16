mod controller;
mod dtos;
mod interceptor;
mod repository;
mod services;

use controller::AuthController;
use serde::Deserialize;
use sword::prelude::*;

pub use dtos::{LoginDto, LoginResponse, RefreshResponse, Session, SessionClaims};
pub use interceptor::SessionCheck;
pub use repository::SessionRepository;
pub use services::{AuthService, LdapClient};

#[config(key = "auth")]
#[derive(Clone, Deserialize)]
pub struct AuthConfig {
    pub ldap_url: String,
    pub ldap_admin_user: String,
    pub ldap_admin_password: String,
    pub ldap_base_dn: String,
    pub access_exp_minutes: i64,
    pub refresh_exp_days: i64,
    pub session_exp_days: i64,
    pub jwt_secret: String,
}

pub struct AuthModule;

impl Module for AuthModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<AuthController>();
    }

    fn register_components(components: &ComponentRegistry) {
        components.register::<AuthService>();
        components.register::<LdapClient>();
        components.register::<SessionRepository>();
    }
}
