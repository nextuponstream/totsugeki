//! Persist data using one of the available database
pub mod inmemory;
pub mod postgresql;

use crate::{
    critical::{DatabaseReadLock, Error as CriticalError},
    parsing::Error as UserInputError,
    resource::Error as ResourceError,
    ApiServiceId, ApiServiceUser, Service,
};
use std::sync::PoisonError;
use thiserror::Error;
use totsugeki::{
    bracket::{
        http_responses::POSTResult, raw::Raw, CreateRequest, Error as BracketError, Id as BracketId,
    },
    join::POSTResponse,
    matches::{Id as MatchId, NextMatchGETResponseRaw, ReportResultPOST},
    organiser::Organiser,
    player::{Participants, GET as PlayersGET},
};

/// Error while persisting data
#[derive(Error, Debug)]
pub enum Error<'a> {
    /// 400: user input could not be parsed
    #[error("Could not parse: {0}")]
    ParseUserInput(#[from] UserInputError),
    /// 403: user action is illegal
    #[error("Action is forbidden:\n\t{0}")]
    Forbidden(ResourceError),
    /// 404: user requested unknown ressource
    #[error("Unable to answer query:\n\t{0}")]
    NotFound(ResourceError),
    /// 500: critical error
    // NOTE: using #[from] macro does not work. Lifetime does not play well
    #[error("Critical error")]
    Critical(CriticalError<'a>),
}

impl<'a> From<CriticalError<'a>> for Error<'a> {
    fn from(e: CriticalError<'a>) -> Self {
        Self::Critical(e)
    }
}

impl<'a> From<ResourceError> for Error<'a> {
    fn from(e: ResourceError) -> Self {
        match e {
            ResourceError::UnknownDiscussionChannel(_)
            | ResourceError::UnknownPlayer
            | ResourceError::UnknownActiveBracketForDiscussionChannel(_)
            | ResourceError::UnknownBracket(_)
            | ResourceError::UnknownResource(_) => Self::NotFound(e),

            ResourceError::ForbiddenBracketUpdate(_) => Self::Forbidden(e),
        }
    }
}

// Bracket... are resources. Then we return a resource error when user is
// trying to query/update the state
impl<'a> From<BracketError> for Error<'a> {
    fn from(e: BracketError) -> Self {
        ResourceError::from(e).into()
    }
}

impl<'a> From<PoisonError<DatabaseReadLock<'a>>> for Error<'a> {
    fn from(e: PoisonError<DatabaseReadLock<'a>>) -> Self {
        CriticalError::from(e).into()
    }
}

/// Modify application state with relevant methods
pub trait DBAccessor {
    /// Clean database to run tests
    ///
    /// # Errors
    /// Returns an error when database could not be cleaned
    fn clean<'a, 'b>(&'a self) -> Result<(), Error<'b>>;

    /// Bar new participants from entering active bracket in discusion channel
    ///
    /// This make it easy to seed bracket
    ///
    /// # Errors
    /// Returns an error if there is no active bracket in channel
    fn close_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
    ) -> Result<BracketId, Error<'c>>;

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

    /// Let TO remove player from running bracket and return id of affected
    /// bracket. Every match against disqualified player is automatically won.
    ///
    /// Uses discussion channel to determine which bracket to update.
    ///
    /// # Errors
    /// Returns an error if the database is unavailable
    fn disqualify_player<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_id: &'b str,
    ) -> Result<BracketId, Error<'c>>;

    /// Find brackets with `bracket_name` filter
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn find_brackets<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        offset: i64,
    ) -> Result<Vec<Raw>, Error<'c>>;

    /// Return next match for this player
    ///
    /// # Errors
    /// Returns an error if there is no match to be played
    fn find_next_match<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service: &'b str,
    ) -> Result<NextMatchGETResponseRaw, Error<'c>>;

    /// Find organisers with `organiser_name` filter
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn find_organisers<'a, 'b, 'c>(
        &'a self,
        organiser_name: &'b str,
        offset: i64,
    ) -> Result<Vec<Organiser>, Error<'c>>;

    /// Let participant forfeit and return id of bracket
    ///
    /// Use discussion channel to determine which bracket to update
    ///
    /// # Errors
    /// Returns an error if the database is unavailable
    fn forfeit<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_internal_id: &'b str,
    ) -> Result<BracketId, Error<'c>>;

    /// Get bracket using id
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn get_bracket<'a, 'b>(&'a self, bracket_id: BracketId) -> Result<Raw, Error<'b>>;

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
        service: &'b str,
    ) -> Result<POSTResponse, Error<'c>>;

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

    /// Returns players in active bracket
    ///
    /// # Errors
    /// Thrown when if discussion channel is unregistered
    fn list_players<'a, 'b>(
        &'a self,
        r: &PlayersGET,
    ) -> Result<(BracketId, Participants), Error<'b>>;

    /// Register service API user
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn list_service_api_user<'a, 'b>(
        &'a self,
        offset: i64,
    ) -> Result<Vec<ApiServiceUser>, Error<'b>>;

    /// Let participant quit bracket before it starts and return id of bracket.
    ///
    /// Use discussion channel to determine which bracket to update
    ///
    /// # Errors
    /// Returns an error if the database is unavailable
    fn quit_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_internal_id: &'b str,
    ) -> Result<BracketId, Error<'c>>;

    /// Register service API user
    ///
    /// # Errors
    /// Returns an error if database could not be accessed
    fn register_service_api_user<'a, 'b, 'c>(
        &'a self,
        service_name: &'b str,
        service_description: &'b str,
    ) -> Result<ApiServiceId, Error<'c>>;

    /// Let TO remove player from bracket before it starts and return id of
    /// affected bracket.
    ///
    /// Uses discussion channel to determine which bracket to update
    ///
    /// # Errors
    /// Returns an error if the database is unavailable
    fn remove_player<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_id: &'b str,
    ) -> Result<BracketId, Error<'c>>;

    /// Let player report result for his active match
    ///
    /// # Errors
    /// Returns an error if result cannot be parsed
    fn report_result<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service: &'b str,
        result: &'b str,
    ) -> Result<ReportResultPOST, Error<'c>>;

    /// Let tournament organiser report result. Has the same effect as both
    /// players reporting the same score.
    ///
    /// # Errors
    /// Returns an error if result cannot be parsed
    fn tournament_organiser_reports_result<'a, 'b, 'c>(
        &'a self,
        channel_internal_id: &'b str,
        service: &'b str,
        player1_id: &'b str,
        result: &'b str,
        player2_id: &'b str,
    ) -> Result<ReportResultPOST, Error<'c>>;

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

    /// Start bracket and allow participants to report result. Use discussion
    /// channel to determine which bracket to start.
    ///
    /// # Errors
    /// Returns an error if the database is unavailable
    fn start_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        // TODO add optionnal player list from name|id to seed bracket
    ) -> Result<BracketId, Error<'c>>;

    /// Let tournament organiser validate match result
    ///
    /// # Errors
    /// Returns an error if result cannot be parsed
    fn validate_result<'a, 'b>(&'a self, match_id: MatchId) -> Result<(), Error<'b>>;
}
