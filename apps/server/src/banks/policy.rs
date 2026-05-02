use crate::banks::*;
use crate::courses::{CourseId, CourseRepository};
use crate::shared::AppResult;
use crate::users::{User, UserRole};

use std::sync::Arc;
use sword::prelude::*;

#[injectable]
pub struct QuestionBankPolicy {
    repository: Arc<QuestionBankRepository>,
    courses: Arc<CourseRepository>,
}

impl QuestionBankPolicy {
    pub async fn require_accessible_course(
        &self,
        current_user: &User,
        course_id: &CourseId,
    ) -> AppResult<()> {
        if self.courses.find_by_id(course_id).await?.is_none() {
            return Err(QuestionBankError::NotFound(course_id.to_string()))?;
        }

        if current_user.role == UserRole::Admin {
            return Ok(());
        }

        if !self.courses.is_member(course_id, &current_user.id).await? {
            return Err(QuestionBankError::Forbidden)?;
        }

        Ok(())
    }

    pub async fn require_accessible_bank(
        &self,
        current_user: &User,
        bank_id: &QuestionBankId,
    ) -> AppResult<QuestionBank> {
        let Some(bank) = self.repository.find_by_id(bank_id).await? else {
            return Err(QuestionBankError::NotFound(bank_id.to_string()))?;
        };

        self.require_accessible_course(current_user, &bank.course_id)
            .await?;

        Ok(bank)
    }
}
