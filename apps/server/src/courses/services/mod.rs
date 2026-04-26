mod policy;

use crate::courses::*;
use crate::shared::{AppResult, TransactionManager};
use crate::users::{User, UserId, UserRepository, UserRole};

use std::sync::Arc;
use sword::prelude::*;

pub use policy::CoursePolicy;

#[injectable]
pub struct CoursesService {
    policy: Arc<CoursePolicy>,
    repository: Arc<CourseRepository>,
    users: Arc<UserRepository>,
    tx: Arc<TransactionManager>,
}

impl CoursesService {
    pub async fn list_for_user(&self, current_user: &User) -> AppResult<Vec<Course>> {
        self.repository.list_for_user(&current_user.id).await
    }

    pub async fn get_one(
        &self,
        current_user: &User,
        course_id: &CourseId,
    ) -> AppResult<CourseView> {
        let course = self
            .policy
            .require_accessible_course(current_user, course_id)
            .await?;

        let members = self.list_course_members(course_id).await?;

        Ok(CourseView::from((&course, &members)))
    }

    pub async fn create(&self, current_user: &User, input: CreateCourseDto) -> AppResult<Course> {
        if !matches!(current_user.role, UserRole::Func | UserRole::Assistant) {
            return Err(CoursesError::Forbidden)?;
        }

        if self.repository.find_by_code(&input.code).await?.is_some() {
            return Err(CoursesError::CodeAlreadyExists)?;
        }

        let course = Course::builder()
            .name(input.name)
            .code(input.code)
            .year(input.year)
            .build();

        let mut tx = self.tx.begin().await?;

        self.repository.save(&mut tx, &course).await?;

        let creator = CourseMember::builder()
            .course_id(course.id)
            .user_id(current_user.id)
            .role(current_user.role.clone())
            .build();

        self.repository.add_member(&mut tx, &creator).await?;

        tx.commit().await?;

        Ok(course)
    }

    pub async fn soft_delete(&self, current_user: &User, course_id: &CourseId) -> AppResult<()> {
        self.policy
            .require_func_member(current_user, course_id)
            .await?;

        if !self.repository.soft_delete(course_id).await? {
            return Err(CoursesError::NotFound(course_id.to_string()))?;
        }

        Ok(())
    }

    pub async fn list_members(
        &self,
        current_user: &User,
        course_id: &CourseId,
    ) -> AppResult<Vec<CourseMemberView>> {
        self.policy
            .require_accessible_course(current_user, course_id)
            .await?;

        self.list_course_members(course_id).await
    }

    pub async fn add_member(
        &self,
        current_user: &User,
        course_id: &CourseId,
        input: AddCourseMemberDto,
    ) -> AppResult<()> {
        self.policy
            .require_func_member(current_user, course_id)
            .await?;

        let target = self
            .users
            .find_by_id(&input.user_id)
            .await?
            .ok_or(CoursesError::MemberNotFound)?;

        if !matches!(target.role, UserRole::Func | UserRole::Assistant) {
            return Err(CoursesError::InvalidMemberRole)?;
        }

        if self.repository.is_member(course_id, &target.id).await? {
            return Err(CoursesError::MemberAlreadyExists)?;
        }

        let course_member = CourseMember::builder()
            .course_id(*course_id)
            .user_id(target.id)
            .role(target.role.clone())
            .build();

        let mut tx = self.tx.begin().await?;

        self.repository.add_member(&mut tx, &course_member).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn remove_member(
        &self,
        current_user: &User,
        course_id: &CourseId,
        user_id: &UserId,
    ) -> AppResult<()> {
        self.policy
            .require_func_member(current_user, course_id)
            .await?;

        self.remove_course_member(course_id, user_id).await
    }

    pub async fn list_course_members(
        &self,
        course_id: &CourseId,
    ) -> AppResult<Vec<CourseMemberView>> {
        self.repository.list_members(course_id).await
    }

    pub async fn remove_course_member(
        &self,
        course_id: &CourseId,
        user_id: &UserId,
    ) -> AppResult<()> {
        let member = self
            .repository
            .find_member(course_id, user_id)
            .await?
            .ok_or(CoursesError::MemberNotFound)?;

        let role_member_count = self
            .repository
            .count_members_by_role(course_id, member.role.clone())
            .await?;

        if member.role == UserRole::Func && role_member_count <= 1 {
            return Err(CoursesError::CannotRemoveLastFuncMember)?;
        }

        if !self.repository.remove_member(course_id, user_id).await? {
            return Err(CoursesError::MemberNotFound)?;
        }

        Ok(())
    }
}
