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

use crate::{
    format::Format, matches::Match, player::Participants, seeding::Method as SeedingMethod,
};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
