//! Bracket domain

mod disqualification;
mod getter_setter;
pub mod late_bracket_configuration;
pub mod matches;
mod ongoing;
pub(crate) mod progression;
mod query_state;
pub mod seeding;
pub(crate) mod winner_bracket;

use crate::player::PlayerID;
use crate::{
    bracket::Id as BracketId,
    format::{Format, ParsingError as FormatParsingError},
    matches::{Error as MatchError, Id as MatchId, Match, MatchParsingError},
    player::{Error as PlayerError, Participants, Player},
    seeding::{
        Error as SeedingError, Method as SeedingMethod, ParsingError as SeedingParsingError,
    },
};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Updating bracket cannot be performed or searched information does not exist
#[derive(Error, Debug)]
pub enum Error {
    /// Error while seeding a bracket
    #[error("{0}")]
    Seeding(#[from] SeedingError),
    /// Error while updating players of bracket
    #[error("{0}")]
    PlayerUpdate(#[from] PlayerError),
    /// Unknown player provided for seeding
    #[error("Unknown player \"{0}\" cannot be used for seeding. Use the following players: {1} of bracket {2}")]
    UnknownPlayer(PlayerID, Participants, BracketId),
    // FIXME remove variant
    /// Cannot add player when they are barred from entering
    #[error("Bracket \"{1}\" does not accept new participants")]
    BarredFromEntering(PlayerID, BracketId),
    /// Bracket has started. Inform user with suggested action.
    #[error("Bracket {0} has started{1}")]
    Started(BracketId, String),
    /// Bracket has not started. Inform user with suggested action.
    #[error("Bracket {0} has not started{1}")]
    NotStarted(BracketId, String),
    /// Player has been disqualified
    #[error("{1} is disqualified\nBracket: {0}")]
    Disqualified(BracketId, Player),
    /// Player has won the tournament and has no match left to play
    #[error("{1} won the tournament and has no matches left to play\nBracket: {0}")]
    NoNextMatch(BracketId, Player),
    /// Player has been eliminated from the tournament
    #[error(
        "{1} has been eliminated from the tournament and has no matches left to play\nBracket: {0}"
    )]
    Eliminated(BracketId, Player),
    /// Player has been eliminated from the tournament
    #[error("{1} is not a participant\nBracket: {0}")]
    PlayerIsNotParticipant(BracketId, Player),
    /// Forbidden action: player has been disqualified
    #[error("{1} is disqualified\nBracket: {0}")]
    ForbiddenDisqualified(BracketId, Player),
    /// No match to play for player
    #[error("There is no matches for you to play\nBracket: {0}")]
    NoMatchToPlay(BracketId, Player),
    /// There is no generated matches at this time
    #[error("No matches were generated yet\nBracket: {0}")]
    NoGeneratedMatches(BracketId),
    /// Tournament is over
    #[error("Tournament is over\nBracket: {0}")]
    TournamentIsOver(BracketId),
    /// Cannot update match
    #[error("{1}\nBracket: {0}")]
    MatchUpdate(BracketId, MatchError),
    /// Referred match is unknown
    #[error("Match {1} is unknown\nBracket: {0}")]
    UnknownMatch(BracketId, MatchId),
    /// Update to match could not happen
    #[error("There is no match to update\nBracket: {0}")]
    NoMatchToUpdate(BracketId, Vec<Match>, MatchId),
}

/// Bracket identifier
pub type Id = Uuid;

/// Add/remove players to the bracket. Then use methods to make the bracket
/// progress.
///
/// Implementation details is delegated to `Progression` struct.
///
/// Seeding is important: <https://youtu.be/ZGoIIV55hEc?t=108>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bracket {
    /// Identifier of this bracket
    id: Id,
    /// Name of this bracket
    name: String,
    /// Players of this bracket
    participants: Participants,
    // FIXME move to instance XXXBracket
    /// Matches from this bracket, sorted by rounds
    matches: Vec<Match>,
    /// Bracket format
    format: Format,
    /// Seeding method used for this bracket
    seeding_method: SeedingMethod,
    /// Advertised start time
    start_time: DateTime<Utc>,
    /// When set to `true`, accept match results
    accept_match_results: bool,
    /// Matches are automatically validated if both players agree on result
    automatic_match_progression: bool,
    /// When set to `true`, bars new participants from entering bracket
    is_closed: bool,
}

/// Error while parsing Bracket
#[derive(Error, Debug)]
pub enum ParsingError {
    /// Could not parse bracket format
    #[error("{0}")]
    Format(#[from] FormatParsingError),
    /// Could not parse seeding method
    #[error("{0}")]
    Seeding(#[from] SeedingParsingError),
    /// Could not parse match
    #[error("{0}")]
    Match(#[from] MatchParsingError),
    /// Could not parse time
    #[error("{0}")]
    Time(#[from] chrono::ParseError),
    /// Could not parse players
    #[error("{0}")]
    Players(#[from] PlayerError),
}

impl std::fmt::Display for Bracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

/// Errors while manipulating bracket
#[derive(Debug)]
pub enum PartitionError {
    /// You need at least 3 players to perform this operation
    NotEnoughPlayersInBracket,
}
