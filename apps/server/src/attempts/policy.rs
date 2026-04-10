use crate::attempts::{AttemptEntity, AttemptError};
use crate::shared::AppResult;
use crate::users::{User, UserRole};

use chrono::Utc;
use sword::prelude::*;

#[injectable]
pub struct AttemptPolicy;

impl AttemptPolicy {
    pub fn can_start_attempt(&self, current_user: &User) -> AppResult<()> {
        if current_user.role == UserRole::Student {
            return Ok(());
        }

        Err(AttemptError::Forbidden.into())
    }

    pub fn can_access_attempt(
        &self,
        current_user: &User,
        attempt: &AttemptEntity,
    ) -> AppResult<()> {
        if attempt.student_id == current_user.id {
            return Ok(());
        }

        Err(AttemptError::Forbidden.into())
    }

    pub fn can_submit_attempt(
        &self,
        current_user: &User,
        attempt: &AttemptEntity,
    ) -> AppResult<()> {
        self.can_access_attempt(current_user, attempt)?;

        if attempt.submitted_at.is_some() {
            return Err(AttemptError::AlreadySubmitted.into());
        }

        if Utc::now() > attempt.expires_at {
            return Err(AttemptError::Expired.into());
        }

        Ok(())
    }

    pub fn can_save_answer(&self, current_user: &User, attempt: &AttemptEntity) -> AppResult<()> {
        self.can_submit_attempt(current_user, attempt)
    }
}
