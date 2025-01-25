//! Convenience types to avoid messy imports like:
//! `use sqlx::error::Error as SqlxError;` x 10
//! when you can alt+enter after typing SqlxError to write
//! `use crate::types::SqlxError;

use axum::extract::State;
use sqlx::{PgPool, Postgres, Transaction};

/// Import type rather than alias all imports of that error
pub type SqlxError = sqlx::error::Error;
/// Database transaction
///
/// Frankly, I'm not onboard to introduce lifetimes. However, it's how I can
/// abstract the SqlxTransaction such that I can reuse that transaction
/// between structs within the same async block.
///
/// Until a better idea comes about, I will stick with this.
pub type SqlxTransaction<'a, 'b> = &'b mut Transaction<'a, Postgres>;

/// Web server framework (axum) holding with database connection pool (sqlx)
pub type ConnectionPool = State<PgPool>;

/// Convenience type
pub type AxumJson<T> = axum::Json<T>;

// /// Convenience type
// pub type StatusCodeAndMessage = Result<(StatusCode, Json<ApiResponse>), Error>;
// pub type StatusCodeAndMessage<T> = Result<(StatusCode, Json<ApiResponse>), T>;
// FAILS
// pub type StatusCodeAndMessage = Result<(StatusCode, Json<ApiResponse>), Error>;
// pub type StatusCodeAndMessage = Result<(StatusCode, Json<ApiResponse>), Error>;
