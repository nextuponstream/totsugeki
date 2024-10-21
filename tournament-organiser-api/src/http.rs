//! Transform library error into http response with error slug
//!
//! For instance, let's say that when joining a bracket, you cannot join again.
//! That would result in a 403 error. Why join again? But in another context,
//! this should result in a 500 error (how did it even happen).

use axum::response::{IntoResponse, Response};
use http::StatusCode;
use sqlx::error::Error as SqlxError;
use std::fmt::Debug;
use thiserror::Error;

/// Return response to user
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error using application
#[derive(Error, Debug)]
pub enum Error {
    /// db error
    #[error("Sqlx error")]
    SqlxError(#[from] SqlxError),
    #[error("Unrecoverable error")]
    /// Unrecoverable error
    Unrecoverable,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl From<Error> for StatusCode {
    fn from(err: Error) -> Self {
        tracing::error!("{err:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// Return status code and slug to user
#[derive(Debug)]
pub(crate) struct ErrorSlug(pub StatusCode, pub Option<String>);

impl ErrorSlug {
    /// I don't like typing `.into()` <https://youtu.be/TGfQu0bQTKc?t=509>
    pub fn new(status_code: StatusCode, slug: impl Into<String>) -> Self {
        Self(status_code, Some(slug.into()))
    }
}

impl From<StatusCode> for ErrorSlug {
    fn from(value: StatusCode) -> Self {
        Self(value, None)
    }
}

impl IntoResponse for ErrorSlug {
    fn into_response(self) -> Response {
        match self {
            ErrorSlug(s, Some(slug)) => (s, slug).into_response(),
            ErrorSlug(s, None) => s.into_response(),
        }
    }
}

/// Log the error and give an opaque response
///
/// Because of the orphan rule, this becomes necessary. Otherwise, you'd do
/// someSqlxOperation?; within a function that returns `AxumResponse`
///
/// `SqlxError` (and other third party library errors) don't implement
/// `IntoResponse` from the axum library. That's why we map the error into a type
/// we control
///
/// We log the error and call it a day. Can't have
///
/// idea: <https://github.com/tokio-rs/axum/blob/52ae7bb904cc374ad0acdc08ae03760a71d95ac2/examples/sqlx-postgres/src/main.rs>
pub(crate) fn internal_error<E: Debug>(err: E) -> ErrorSlug {
    tracing::error!("{err:?}");
    ErrorSlug::new(StatusCode::INTERNAL_SERVER_ERROR, "internal-error")
}

impl From<ErrorSlug> for Error {
    fn from(value: ErrorSlug) -> Self {
        tracing::error!("{value:?}");
        Self::Unrecoverable
    }
}
