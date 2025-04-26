use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

pub type QrLinkResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(rusqlite::Error),

    #[error("Lock poisoned: {0}")]
    LockError(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status_code, message) = match &self {
            Error::DatabaseError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error))
            }
            Error::LockError(error) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error)),
        };

        (status_code, message).into_response()
    }
}

impl From<Error> for String {
    fn from(value: Error) -> Self {
        match &value {
            Error::DatabaseError(error) => format!("{}", error),
            Error::LockError(error) => error.to_owned(),
        }
    }
}
