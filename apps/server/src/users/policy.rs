use crate::shared::AppResult;
use crate::users::{User, UserRole, UsersError};

use sword::prelude::*;

#[injectable]
pub struct UserPolicy;

impl UserPolicy {
    pub fn can_assign_assistant_role(&self, target: &User) -> AppResult<()> {
        if target.role == UserRole::Func {
            return Err(UsersError::InvalidAssistantTargetRole)?;
        }

        Ok(())
    }
}
