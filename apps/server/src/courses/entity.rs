use crate::{
    shared::{Entity, Id},
    users::{UserId, UserRole},
};

use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub type CourseId = Id<Course>;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow, Builder)]
pub struct Course {
    #[builder(default = CourseId::new())]
    pub id: CourseId,
    pub name: String,
    pub code: String,
    pub year: i16,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Entity for Course {
    fn key_name() -> &'static str {
        "course"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow, Builder)]
#[serde(rename_all = "camelCase")]
pub struct CourseMember {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub course_id: CourseId,
    pub user_id: UserId,
    pub role: UserRole,
}
