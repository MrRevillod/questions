use crate::courses::{Course, CourseId, CourseRepository, CoursesError};
use crate::shared::AppResult;
use crate::users::User;

use sword::prelude::*;

#[injectable]
pub struct CoursePolicy {
    repository: CourseRepository,
}

impl CoursePolicy {
    pub async fn require_accessible_course(
        &self,
        current_user: &User,
        course_id: &CourseId,
    ) -> AppResult<Course> {
        let Some(course) = self.repository.find_by_id(course_id).await? else {
            return Err(CoursesError::NotFound(course_id.to_string()))?;
        };

        if !self
            .repository
            .is_member(&course.id, &current_user.id)
            .await?
        {
            return Err(CoursesError::Forbidden)?;
        }

        Ok(course)
    }

    pub async fn require_func_member(
        &self,
        current_user: &User,
        course_id: &CourseId,
    ) -> AppResult<Course> {
        let course = self
            .require_accessible_course(current_user, course_id)
            .await?;

        if !self
            .repository
            .is_func_member(course_id, &current_user.id)
            .await?
        {
            return Err(CoursesError::OnlyFuncCanManageMembers)?;
        }

        Ok(course)
    }
}
