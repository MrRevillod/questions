use crate::courses::{Course, CourseId, CourseMember, CourseMemberView};
use crate::shared::{AppResult, Database, Tx};
use crate::users::{UserId, UserRole};

use chrono::Utc;
use std::sync::Arc;
use sword::prelude::*;

#[injectable]
pub struct CourseRepository {
    db: Arc<Database>,
}

impl CourseRepository {
    pub async fn find_by_id(&self, course_id: &CourseId) -> AppResult<Option<Course>> {
        let course = sqlx::query_as::<_, Course>(
            "SELECT * FROM courses WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(course_id)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(course)
    }

    pub async fn find_by_code(&self, code: &str) -> AppResult<Option<Course>> {
        let course = sqlx::query_as::<_, Course>(
            "SELECT * FROM courses WHERE code = $1 AND deleted_at IS NULL",
        )
        .bind(code)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(course)
    }

    pub async fn list_for_user(&self, user_id: &UserId) -> AppResult<Vec<Course>> {
        let courses = sqlx::query_as::<_, Course>(
            "SELECT c.*
             FROM courses c
             INNER JOIN course_members cm ON cm.course_id = c.id
             WHERE cm.user_id = $1
               AND c.deleted_at IS NULL
             ORDER BY c.year DESC, c.name ASC",
        )
        .bind(user_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(courses)
    }

    pub async fn create(&self, tx: &mut Tx<'_>, course: &Course) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO courses (id, name, code, year, deleted_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(course.id)
        .bind(&course.name)
        .bind(&course.code)
        .bind(course.year)
        .bind(course.deleted_at)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn soft_delete(&self, course_id: &CourseId) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE courses SET deleted_at = $2
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(course_id)
        .bind(Utc::now())
        .execute(self.db.get_pool())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn is_member(&self, course_id: &CourseId, user_id: &UserId) -> AppResult<bool> {
        let is_member = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
            	SELECT 1 FROM course_members WHERE course_id = $1 AND user_id = $2
            )",
        )
        .bind(course_id)
        .bind(user_id)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(is_member)
    }

    pub async fn is_func_member(&self, course_id: &CourseId, user_id: &UserId) -> AppResult<bool> {
        let is_func_member = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM course_members WHERE course_id = $1 AND user_id = $2 AND role = 'func'
            )",
        )
        .bind(course_id)
        .bind(user_id)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(is_func_member)
    }

    pub async fn add_member(&self, tx: &mut Tx<'_>, course_member: &CourseMember) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO course_members (id, course_id, user_id, role)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(course_member.id)
        .bind(course_member.course_id)
        .bind(course_member.user_id)
        .bind(course_member.role.clone())
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn remove_member(&self, course_id: &CourseId, user_id: &UserId) -> AppResult<bool> {
        let result =
            sqlx::query("DELETE FROM course_members WHERE course_id = $1 AND user_id = $2")
                .bind(course_id)
                .bind(user_id)
                .execute(self.db.get_pool())
                .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_member(
        &self,
        course_id: &CourseId,
        user_id: &UserId,
    ) -> AppResult<Option<CourseMember>> {
        let member = sqlx::query_as::<_, CourseMember>(
            "SELECT * FROM course_members WHERE course_id = $1 AND user_id = $2",
        )
        .bind(course_id)
        .bind(user_id)
        .fetch_optional(self.db.get_pool())
        .await?;

        Ok(member)
    }

    pub async fn count_members_by_role(
        &self,
        course_id: &CourseId,
        role: UserRole,
    ) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM course_members WHERE course_id = $1 AND role = $2",
        )
        .bind(course_id)
        .bind(role)
        .fetch_one(self.db.get_pool())
        .await?;

        Ok(count)
    }

    pub async fn list_members(&self, course_id: &CourseId) -> AppResult<Vec<CourseMemberView>> {
        let members = sqlx::query_as::<_, CourseMemberView>(
            "SELECT u.id AS user_id, u.username, u.name, cm.role
             FROM course_members cm
             INNER JOIN users u ON u.id = cm.user_id
             WHERE cm.course_id = $1
             ORDER BY u.username ASC",
        )
        .bind(course_id)
        .fetch_all(self.db.get_pool())
        .await?;

        Ok(members)
    }
}
