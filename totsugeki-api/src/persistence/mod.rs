//! Persist data using one of the available database
pub mod inmemory;
pub mod postgresql;

use crate::matches::NextMatchGET;
use crate::{ApiServiceId, ApiServiceUser, Service};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};
use totsugeki::matches::MatchResultParsingError;
use totsugeki::{
    bracket::{Bracket, FormatParsingError, Id as BracketId, POSTResult},
    join::POSTResponseBody,
    matches::{Error as MatchError, Id as MatchId},
    organiser::Organiser,
    seeding::{Error as SeedingError, ParsingError as SeedingParsingError},
};

/// Error while parsing ``InteralIdType`` of service used
#[derive(Debug)]
pub enum ParseServiceInternalIdError {
    /// Parsing error
    Parse(String),
}

impl FromStr for Service {
    type Err = ParseServiceInternalIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "discord" => Ok(Self::Discord),
            _ => Err(ParseServiceInternalIdError::Parse(format!(
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
    Denied(String),
    /// Parsing error
    Parsing(String),
    /// Unknown
    Unknown(String),
    /// Bracket was not found
    BracketNotFound(BracketId),
    /// Discussion channel was not found
    DiscussionChannelNotFound,
    /// No active bracket for provided discussion channel
    NoActiveBracketInDiscussionChannel,
    /// Player was not found
    PlayerNotFound,
    /// Next match was not found for this bracket
    NextMatchNotFound,
    /// There is either not enough players in bracket or the player's run in bracket has ended
    NoNextMatch,
    /// To many players causes math overflow
    Seeding(SeedingError),
    /// No opponent was found to report result against
    NoOpponent,
    /// Match could not be updated
    Match(MatchError),
    /// Player searched for his next match in bracket but his was eliminated
    EliminatedFromBracket,
}

impl<'a> From<MatchResultParsingError> for Error<'a> {
    fn from(e: MatchResultParsingError) -> Self {
        Self::Parsing(e.to_string())
    }
}

impl<'a> From<MatchError> for Error<'a> {
    fn from(e: MatchError) -> Self {
        Self::Match(e)
    }
}

impl<'a> From<SeedingError> for Error<'a> {
    fn from(e: totsugeki::seeding::Error) -> Self {
        Self::Seeding(e)
    }
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
            Error::Denied(msg) => writeln!(f, "Reason: {msg}"),
            Error::PoisonedReadLock(e) => e.fmt(f),
            Error::PoisonedWriteLock(e) => e.fmt(f),
            Error::BracketNotFound(b_id) => writeln!(f, "Bracket not found: {b_id}"),
            Error::DiscussionChannelNotFound => writeln!(
                f,
                "Discussion channel not found because it is not registered"
            ),
            Error::NoActiveBracketInDiscussionChannel => {
                writeln!(f, "There is no active bracket in this discussion channel")
            }
            Error::PlayerNotFound => writeln!(f, "Player is not registered"),
            Error::NextMatchNotFound => writeln!(f, "Next match was not found"),
            Error::NoNextMatch => write!(f, "There is no match for you to play."),
            Error::Seeding(e) => e.fmt(f),
            Error::NoOpponent => write!(f, "No opponent"),
            Error::Match(_e) => write!(f, "Match could not be updated"),
            Error::EliminatedFromBracket => write!(f, "There is no match for you to play because you have been eliminated from the bracket."),
        }
    }
}

impl<'a> From<SeedingParsingError> for Error<'a> {
    fn from(e: SeedingParsingError) -> Self {
        Error::Parsing(format!("{e:?}"))
    }
}

impl<'a> From<FormatParsingError> for Error<'a> {
    fn from(e: FormatParsingError) -> Self {
        Error::Parsing(format!("{e:?}"))
    }
}

impl<'a> From<uuid::Error> for Error<'a> {
    fn from(e: uuid::Error) -> Self {
        Error::Parsing(format!("Uuid could not be parsed: {e}")) // TODO better error propagation
    }
}

/// Parameters to create a bracket
pub struct BracketRequest<'b> {
    /// requested bracket name
    pub bracket_name: &'b str,
    /// requested bracket format
    pub bracket_format: &'b str,
    /// seeding method of requested bracket
    pub seeding_method: &'b str,
    /// Organiser name of requested bracket
    pub organiser_name: &'b str,
    /// Organiser id of requested bracket while using service
    pub organiser_internal_id: &'b str,
    /// Id of internal channel
    pub internal_channel_id: &'b str,
    /// Type of service used to make request
    pub service_type_id: &'b str,
}

impl<'a> From<totsugeki::player::Error> for Error<'a> {
    fn from(e: totsugeki::player::Error) -> Self {
        Self::Parsing(format!("could not form players group: {e:?}")) // FIXME don't use string
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
    fn create_bracket<'a, 'b, 'c>(&'a self, r: BracketRequest<'b>)
        -> Result<POSTResult, Error<'c>>;

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

    /// Join bracket as a player
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn join_bracket<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        player_name: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<POSTResponseBody, Error<'c>>;

    /// Get bracket using id
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn get_bracket<'a, 'b>(&'a self, bracket_id: BracketId) -> Result<Bracket, Error<'b>>;

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
    fn list_service_api_user<'a, 'b>(
        &'a self,
        offset: i64,
    ) -> Result<Vec<ApiServiceUser>, Error<'b>>;

    /// Register service API user
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn register_service_api_user<'a, 'b, 'c>(
        &'a self,
        service_name: &'b str,
        service_description: &'b str,
    ) -> Result<ApiServiceId, Error<'c>>;

    /// Return next match for this player
    ///
    /// # Errors
    /// Returns an error if there is no match to be played
    fn find_next_match<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<NextMatchGET, Error<'c>>;

    /// Let player report result for his active match
    ///
    /// # Errors
    /// Returns an error if result cannot be parsed
    fn report_result<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
        result: &'b str,
    ) -> Result<MatchId, Error<'c>>;

    /// Let tournament organiser validate match result
    ///
    /// # Errors
    /// Returns an error if result cannot be parsed
    fn validate_result<'a, 'b>(&'a self, match_id: MatchId) -> Result<(), Error<'b>>;
}
