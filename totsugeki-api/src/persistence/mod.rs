//! Persist data using one of the available database
pub mod inmemory;
pub mod postgresql;

use crate::{ApiServiceId, ApiServiceUser, Service};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};
use totsugeki::matches::MatchResultParsingError;
use totsugeki::DiscussionChannelId;
use totsugeki::{
    bracket::{
        CreateRequest, Error as BracketError, Id as BracketId, POSTResult,
        ParsingError as BracketParsingError, Raw,
    },
    format::ParsingError as FormatParsingError,
    join::POSTResponseBody,
    matches::{Id as MatchId, NextMatchGETResponseRaw},
    organiser::Organiser,
    player::{Players, GET as PlayersGET},
    seeding::ParsingError as SeedingParsingError,
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
    /// User was denied access to ressource
    Denied(String),
    /// Parsing error
    Parsing(String),
    /// Data is not present when it should be
    Corrupted(String),
    /// Bracket is not registered
    UnregisteredBracket(BracketId),
    /// Discussion channel for `Service` using provided id is not registered
    UnregisteredDiscussionChannel(Service, String),
    /// User searched for active bracket in discussion channel but there was
    /// none to be found
    NoActiveBracketInDiscussionChannel(DiscussionChannelId),
    /// Player was not found
    UnregisteredPlayer,
    /// Player searched for his next match in bracket but his was eliminated
    EliminatedFromBracket,
    /// Cannot update bracket
    UpdateBracket(BracketError),
    /// Query cannot be answered for bracket
    BadBracketQuery(BracketError),
    /// Unknown match
    UnknownMatch(MatchId),
}

impl<'a> From<BracketParsingError> for Error<'a> {
    fn from(e: BracketParsingError) -> Self {
        Self::Parsing(format!("Could not parse bracket: {e}"))
    }
}

impl<'a> From<BracketError> for Error<'a> {
    fn from(e: BracketError) -> Self {
        match e {
            BracketError::Seeding(_)
            | BracketError::MissingArgument(_, _)
            | BracketError::PlayerUpdate(_)
            | BracketError::UnknownMatch(_)
            | BracketError::Match(_)
            | BracketError::UnknownPlayer(_, _)
            | BracketError::Players(_, _)
            | BracketError::BarredFromEntering(_, _)
            | BracketError::AcceptResults(_, _)
            | BracketError::NoMatchToPlay(_, _) => Self::UpdateBracket(e),
            BracketError::NoNextMatch(_)
            | BracketError::NoGeneratedMatches
            | BracketError::EliminatedFromBracket(_)
            | BracketError::PlayerIsNotParticipant(_, _) => Self::BadBracketQuery(e),
        }
    }
}

impl<'a> From<MatchResultParsingError> for Error<'a> {
    fn from(e: MatchResultParsingError) -> Self {
        Self::Parsing(format!("Match result: {e}"))
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
            Error::Corrupted(msg) | Error::Parsing(msg) => writeln!(f, "{msg}"),
            Error::Denied(msg) => writeln!(f, "Reason: {msg}"),
            Error::PoisonedReadLock(e) => e.fmt(f),
            Error::PoisonedWriteLock(e) => e.fmt(f),
            Error::UnregisteredBracket(b_id) => writeln!(f, "Bracket \"{b_id}\" is unregistered"),
            Error::UnregisteredDiscussionChannel(_, _) => writeln!(
                f,
                "Discussion channel is not registered"
            ),
            Error::NoActiveBracketInDiscussionChannel(id) => {
                writeln!(f, "There is no active bracket in discussion channel ({id})")
            }
            Error::UnregisteredPlayer => writeln!(f, "Player is not registered"),
            Error::EliminatedFromBracket => write!(f, "There is no match for you to play because you have been eliminated from the bracket."),
            Error::UpdateBracket(e) => write!(f, "Bracket cannot be updated: {e}"),
            Error::BadBracketQuery(e) => write!(f, "Unable to answer query: {e}"),
            Error::UnknownMatch(id) => write!(f, "Unknown match: {id}"),
        }
    }
}

impl<'a> From<SeedingParsingError> for Error<'a> {
    fn from(e: SeedingParsingError) -> Self {
        Error::Parsing(format!("Could not parse seed: {e:?}"))
    }
}

impl<'a> From<FormatParsingError> for Error<'a> {
    fn from(e: FormatParsingError) -> Self {
        Error::Parsing(format!("Could not parse bracket: {e:?}"))
    }
}

impl<'a> From<uuid::Error> for Error<'a> {
    fn from(e: uuid::Error) -> Self {
        Error::Parsing(format!("Uuid could not be parsed: {e}")) // TODO better error propagation
    }
}

impl<'a> From<totsugeki::player::Error> for Error<'a> {
    fn from(e: totsugeki::player::Error) -> Self {
        Self::Parsing(format!("could not form players group: {e:?}")) // FIXME don't use string
    }
}

/// Datase underlying the api
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
    fn create_bracket<'a, 'b, 'c>(&'a self, r: CreateRequest<'b>) -> Result<POSTResult, Error<'c>>;

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
    ) -> Result<Vec<Raw>, Error<'c>>;

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
    fn get_bracket<'a, 'b>(&'a self, bracket_id: BracketId) -> Result<Raw, Error<'b>>;

    /// List brackets
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn list_brackets<'a, 'b>(&'a self, offset: i64) -> Result<Vec<Raw>, Error<'b>>;

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

    /// Returns players in active bracket
    ///
    /// # Errors
    /// Thrown when if discussion channel is unregistered
    fn list_players<'a, 'b>(&'a self, r: &PlayersGET) -> Result<(BracketId, Players), Error<'b>>;

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
    ) -> Result<NextMatchGETResponseRaw, Error<'c>>;

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

    /// Start bracket and allow participants to report result. Use discussion
    /// channel to determine which bracket to start.
    ///
    /// # Errors
    /// Returns an error if the database is unavailable
    fn start_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service_type_id: &'b str,
        // TODO add optionnal player list from name|id to seed bracket
    ) -> Result<BracketId, Error<'c>>;

    /// Bar new participants from entering active bracket in discusion channel
    ///
    /// This make it easy to seed bracket
    ///
    /// # Errors
    /// Returns an error if there is no active bracket in channel
    fn bar_from_entering_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<BracketId, Error<'c>>;

    /// Seed active bracket in discussion channel
    ///
    /// # Errors
    /// Returns an error if there is no bracket to seed
    fn seed_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        players: Vec<String>,
    ) -> Result<BracketId, Error<'c>>;
}
