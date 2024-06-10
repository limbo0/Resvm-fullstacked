use http::status::StatusCode;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum MyUsersError {
    #[error("Not Found")]
    NotFound,
    #[error("Internal Server Error")]
    InternalServerError,
}

impl MyUsersError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            MyUsersError::NotFound => StatusCode::NOT_FOUND,
            MyUsersError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
