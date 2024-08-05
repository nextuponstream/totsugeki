//! Transform library error into http response with error slug
//!
//! For instance, let's say that when joining a bracket, you cannot join again.
//! That would result in a 403 error. Why join again? But in another context,
//! this should result in a 500 error (how did it even happen).

use axum::response::{IntoResponse, Response};
use http::StatusCode;
use sqlx::error::Error as SqlxError;
use thiserror::Error;

/// Return response to user
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error using application
#[derive(Error, Debug)]
pub enum Error {
    /// db error
    #[error("Sqlx error")]
    SqlxError(#[from] SqlxError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
