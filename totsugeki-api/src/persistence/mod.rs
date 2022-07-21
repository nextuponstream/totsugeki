//! Persist data using one of the available database
pub mod inmemory;
pub mod postgresql;

use crate::{ApiServiceId, ApiServiceUser, BracketPOSTResult, InternalIdType};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};
use totsugeki::{bracket::Bracket, organiser::Organiser};

/// Error while parsing InteralIdType
#[derive(Debug)]
pub enum ParseInternalIdTypeError {
    /// Parsing error
    Parse(String),
}

impl FromStr for InternalIdType {
    type Err = ParseInternalIdTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "discord" => Ok(Self::Discord),
            _ => Err(ParseInternalIdTypeError::Parse(format!(
                "could not parse {s}"
            ))),
        }
    }
}

/// Read lock to database
pub type DatabaseReadLock<'a> = RwLockReadGuard<'a, Box<dyn DBAccessor + Send + Sync>>;
/// Write lock to database
pub type DatabaseWriteLock<'a> = RwLockWriteGuard<'a, Box<dyn DBAccessor + Send + Sync>>;

/// Error while persisting data
#[derive(Debug)]
pub enum Error<'a> {
    /// Lock to the database is poisoned when attempting to read
    PoisonedReadLock(PoisonError<DatabaseReadLock<'a>>),
    /// Lock to the database is poisoned when attempting to write
    PoisonedWriteLock(PoisonError<DatabaseWriteLock<'a>>),
    /// Database error with error code
    Code(String),
    /// Denied access
    Denied(),
    /// Parsing error
    Parsing(String),
    /// Unknown
    Unknown(String),
}

impl<'a> From<PoisonError<DatabaseReadLock<'a>>> for Error<'a> {
    fn from(e: PoisonError<DatabaseReadLock<'a>>) -> Self {
        Error::PoisonedReadLock(e)
    }
}

impl<'a> From<PoisonError<DatabaseWriteLock<'a>>> for Error<'a> {
    fn from(e: PoisonError<DatabaseWriteLock<'a>>) -> Self {
        Error::PoisonedWriteLock(e)
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Code(msg) | Error::Unknown(msg) | Error::Parsing(msg) => writeln!(f, "{msg}"),
            Error::Denied() => writeln!(f, "Denied"),
            Error::PoisonedReadLock(e) => e.fmt(f),
            Error::PoisonedWriteLock(e) => e.fmt(f),
        }
    }
}

/// Datase underlying a tournament server
pub trait DBAccessor {
    /// Clean database to run tests
    ///
    /// # Errors
    /// Returns an error when database could not be cleaned
    fn clean<'a, 'b>(&'a self) -> Result<(), Error<'b>>;

    /// Create bracket with name `bracket_name`. If organiser is not know, create organiser with name `organiser_name`
    ///
    /// # Errors
    /// Returns error if bracket could not be persisted
    fn create_bracket<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        organiser_name: &'b str,
        organiser_internal_id: String,
        internal_channel_id: String,
        internal_id_type: InternalIdType,
    ) -> Result<BracketPOSTResult, Error<'c>>;

    /// Create tournament organiser
    ///
    /// # Errors
    /// Returns an error if tournament organiser could not be persisted
    fn create_organiser<'a, 'b, 'c>(&'a self, organiser_name: &'b str) -> Result<(), Error<'c>>;

    /// Find brackets with `bracket_name` filter
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn find_brackets<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        offset: i64,
    ) -> Result<Vec<Bracket>, Error<'c>>;

    /// Find organisers with `organiser_name` filter
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn find_organisers<'a, 'b, 'c>(
        &'a self,
        organiser_name: &'b str,
        offset: i64,
    ) -> Result<Vec<Organiser>, Error<'c>>;

    /// Initialize database if no database is present
    ///
    /// # Errors
    /// Returns an error when database failed to initialize.
    fn init(&self) -> Result<(), Error>;

    /// List brackets
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn list_brackets<'a, 'b>(&'a self, offset: i64) -> Result<Vec<Bracket>, Error<'b>>;

    /// List organisers
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn list_organisers<'a, 'b>(&'a self, offset: i64) -> Result<Vec<Organiser>, Error<'b>>;

    /// Register service API user
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn list_service_api_user<'a, 'b, 'c>(
        &'a self,
        offset: i64,
    ) -> Result<Vec<ApiServiceUser>, Error<'c>>;

    /// Register service API user
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn register_service_api_user<'a, 'b, 'c>(
        &'a self,
        service_name: &'b str,
        service_description: &'b str,
    ) -> Result<ApiServiceId, Error<'c>>;
}
