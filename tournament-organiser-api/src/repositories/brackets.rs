//! Bracket repository

use crate::types::SqlxError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use totsugeki_core::matches::Match;
use tracing::error;

/// All errors when joining a bracket
#[derive(Error, Debug)]
pub enum Error {
    #[error("Unrecoverable database error")]
    /// Error with postgres, unrecoverable
    Sqlx(SqlxError),
    /// Inconsistent state in the client
    #[error("player tried to join bracket but they are already in")]
    PlayerAlreadyPresent,
}

impl From<SqlxError> for Error {
    fn from(err: SqlxError) -> Self {
        Self::Sqlx(err)
    }
}

/// Matches raw value
#[derive(Deserialize, Serialize)]
pub struct MatchesRaw(pub Vec<Match>);
