use sword::web::HttpError;
use thiserror::Error;

#[derive(Debug, Error, HttpError)]
pub enum CoursesError {
    #[http(code = 404, message = "The requested course was not found.")]
    #[error("Course not found: {0}")]
    NotFound(String),

    #[http(code = 400, message = "The provided course ID is invalid.")]
    #[error("Invalid course ID")]
    InvalidId,

    #[http(code = 403, message = "You do not have access to this course.")]
    #[error("Forbidden course access")]
    Forbidden,

    #[http(
        code = 403,
        message = "Only func course members can manage course members."
    )]
    #[error("Only func members can manage course members")]
    OnlyFuncCanManageMembers,

    #[http(code = 409, message = "A course with this code already exists.")]
    #[error("Course code already exists")]
    CodeAlreadyExists,

    #[http(code = 409, message = "This user is already a member of this course.")]
    #[error("Course member already exists")]
    MemberAlreadyExists,

    #[http(code = 404, message = "The requested course member was not found.")]
    #[error("Course member not found")]
    MemberNotFound,

    #[http(
        code = 400,
        message = "Only assistant and func users can be course members."
    )]
    #[error("Invalid course member role")]
    InvalidMemberRole,

    #[http(
        code = 409,
        message = "You cannot remove the last func member from a course."
    )]
    #[error("Cannot remove last func member")]
    CannotRemoveLastFuncMember,
}
