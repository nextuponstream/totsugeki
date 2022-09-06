//! Critical error encountered
use crate::DBAccessor;
use std::sync::{PoisonError, RwLockReadGuard};
use thiserror::Error;

/// Critical errors during api lifetime
#[derive(Error, Debug)]
pub enum Error<'a> {
    /// Shared lock over database info is unusable
    #[error("")]
    PoisonedReadLock(PoisonError<DatabaseReadLock<'a>>),
    /// Data is not present when it should
    #[error("")]
    Corrupted(String),
}

/// Read lock to shared database
pub type DatabaseReadLock<'a> = RwLockReadGuard<'a, Box<dyn DBAccessor + Send + Sync>>;

impl<'a> From<PoisonError<DatabaseReadLock<'a>>> for Error<'a> {
    fn from(e: PoisonError<DatabaseReadLock<'a>>) -> Self {
        Error::PoisonedReadLock(e)
    }
}
