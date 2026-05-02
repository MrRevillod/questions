use crate::auth::SessionCheck;
use crate::authz::{AuthzAction, AuthzGuard};
use crate::courses::*;
use crate::shared::RequestExt;
use crate::users::UserId;

use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/courses")]
#[interceptor(SessionCheck)]
pub struct CoursesController {
    courses: Arc<CoursesService>,
}

impl CoursesController {
    #[get("/")]
    #[interceptor(AuthzGuard, config = AuthzAction::CourseList)]
    pub async fn get_courses(&self, req: Request) -> WebResult {
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;
        let courses = self.courses.list_for_user(current_user).await?;

        Ok(JsonResponse::Ok().data(courses))
    }

    #[get("/{courseId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::CourseRead)]
    pub async fn get_course(&self, req: Request) -> WebResult {
        let course_id = req.param::<CourseId>("courseId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        let course = self.courses.get_one(current_user, &course_id).await?;

        Ok(JsonResponse::Ok().data(course))
    }

    #[post("/")]
    #[interceptor(AuthzGuard, config = AuthzAction::CourseCreate)]
    pub async fn create_course(&self, req: Request) -> WebResult {
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;
        let input = req.body_validator::<CreateCourseDto>()?;

        let course = self.courses.create(current_user, input).await?;

        Ok(JsonResponse::Created().data(course))
    }

    #[delete("/{courseId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::CourseDelete)]
    pub async fn delete_course(&self, req: Request) -> WebResult {
        let course_id = req.param::<CourseId>("courseId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        self.courses.soft_delete(current_user, &course_id).await?;

        Ok(JsonResponse::Ok().message("Course deleted successfully"))
    }

    #[get("/{courseId}/members")]
    #[interceptor(AuthzGuard, config = AuthzAction::CourseRead)]
    pub async fn list_members(&self, req: Request) -> WebResult {
        let course_id = req.param::<CourseId>("courseId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        let members = self.courses.list_members(current_user, &course_id).await?;

        Ok(JsonResponse::Ok().data(members))
    }

    #[post("/{courseId}/members")]
    #[interceptor(AuthzGuard, config = AuthzAction::CourseManageMembers)]
    pub async fn add_member(&self, req: Request) -> WebResult {
        let course_id = req.param::<CourseId>("courseId")?;
        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;
        let input = req.body_validator::<AddCourseMemberDto>()?;

        self.courses
            .add_member(current_user, &course_id, input)
            .await?;

        Ok(JsonResponse::Ok().message("Course member added successfully"))
    }

    #[delete("/{courseId}/members/{userId}")]
    #[interceptor(AuthzGuard, config = AuthzAction::CourseManageMembers)]
    pub async fn remove_member(&self, req: Request) -> WebResult {
        let course_id = req.param::<CourseId>("courseId")?;
        let user_id = req.param::<UserId>("userId")?;

        let current_user = req.user().ok_or_else(JsonResponse::Unauthorized)?;

        self.courses
            .remove_member(current_user, &course_id, &user_id)
            .await?;

        Ok(JsonResponse::Ok().message("Course member removed successfully"))
    }
}
