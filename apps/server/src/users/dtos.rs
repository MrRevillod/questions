use std::str::FromStr;

use crate::users::{UserFilter, UserRole, UsersError};
use serde::Deserialize;
use validator::Validate;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ManageableUserRole {
    Student,
    Assistant,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRoleRequest {
    pub role: ManageableUserRole,
}

#[derive(Debug, Default, Deserialize)]
pub struct SearchUsersQuery {
    pub search: Option<String>,
    pub roles: Option<String>,
}

impl From<ManageableUserRole> for UserRole {
    fn from(value: ManageableUserRole) -> Self {
        match value {
            ManageableUserRole::Student => Self::Student,
            ManageableUserRole::Assistant => Self::Assistant,
        }
    }
}

impl FromStr for UserRole {
    type Err = UsersError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "student" => Ok(Self::Student),
            "func" => Ok(Self::Func),
            "assistant" => Ok(Self::Assistant),
            "admin" => Ok(Self::Admin),
            _ => Err(UsersError::InvalidUserRole),
        }
    }
}

impl From<SearchUsersQuery> for UserFilter {
    fn from(value: SearchUsersQuery) -> Self {
        let roles = value.roles.as_ref().map(|roles| {
            roles
                .split(',')
                .filter_map(|r| UserRole::from_str(r.trim()).ok())
                .collect::<Vec<_>>()
        });

        Self {
            search: value.search,
            roles,
        }
    }
}
