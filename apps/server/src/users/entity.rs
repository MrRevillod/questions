use crate::shared::{Entity, Id};
use bon::Builder;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Student,
    Func,
    Assistant,
    Admin,
}

pub type UserId = Id<User>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FromRow, Builder)]
pub struct User {
    #[builder(default = UserId::new())]
    pub id: UserId,
    pub username: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
}

impl Entity for User {
    fn key_name() -> &'static str {
        "user"
    }
}

#[derive(Debug, Clone)]
pub struct UserFilter {
    pub search: Option<String>,
    pub roles: Option<Vec<UserRole>>,
}
