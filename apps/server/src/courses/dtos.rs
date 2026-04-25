use crate::users::UserId;

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateCourseDto {
    #[validate(length(
        min = 1,
        max = 120,
        message = "Name must be between 1 and 120 characters"
    ))]
    pub name: String,
    #[validate(length(
        min = 2,
        max = 32,
        message = "Code must be between 2 and 32 characters"
    ))]
    pub code: String,

    #[validate(range(min = 2000, max = 2100, message = "Year must be between 2000 and 2100"))]
    pub year: i16,
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AddCourseMemberDto {
    pub user_id: UserId,
}
