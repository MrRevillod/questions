use crate::{
    courses::{Course, CourseId},
    users::{UserId, UserRole},
};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CourseMemberView {
    pub user_id: UserId,
    pub username: String,
    pub name: String,
    pub role: UserRole,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseView {
    pub id: CourseId,
    pub name: String,
    pub code: String,
    pub year: i16,
    pub members: Vec<CourseMemberView>,
}

impl From<(&Course, &Vec<CourseMemberView>)> for CourseView {
    fn from((course, members): (&Course, &Vec<CourseMemberView>)) -> Self {
        Self {
            id: course.id,
            name: course.name.clone(),
            code: course.code.clone(),
            year: course.year,
            members: members.clone(),
        }
    }
}
