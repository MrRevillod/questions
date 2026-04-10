use crate::authz::AuthzAction;

use sword::web::HttpError;
use thiserror::Error;

#[derive(Debug, Error, HttpError)]
pub enum AuthzError {
    #[http(code = 403, message = "You do not have permission for this action.")]
    #[error("Forbidden action: {0:?}")]
    Forbidden(AuthzAction),

    #[http(code = 401, message = "Authenticated user could not be resolved.")]
    #[error("Authenticated actor not found: {0}")]
    ActorNotFound(String),
}
