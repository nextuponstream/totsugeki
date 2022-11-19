//! Bracket domain

mod disqualification;
mod getter_setter;
pub mod http_responses;
pub mod matches;
mod participants;
mod progression;
mod query_state;
pub mod raw;
mod seeding;

use crate::{
    bracket::{matches::Error as ProgressError, Id as BracketId},
    format::{Format, ParsingError as FormatParsingError},
    matches::{Id as MatchId, Match, MatchParsingError},
    player::{Error as PlayerError, Id as PlayerId, Participants},
    seeding::{
        Error as SeedingError, Method as SeedingMethod, ParsingError as SeedingParsingError,
    },
    DiscussionChannelId,
};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

/// Updating bracket cannot be performed or searched information does not exist
// FIXME try reducing this error size under 136 bytes
// FIXME find current error size
// Fix conceptual error for delegating error:
// bracket cannot update if it has not started, then this error is handled at
// the bracket let, not in a ProgressionError
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
    UnknownPlayer(PlayerId, Participants, BracketId),
    /// Cannot add player when they are barred from entering
    #[error("Bracket \"{1}\" does not accept new participants")]
    BarredFromEntering(PlayerId, BracketId),
    /// Bracket has started. Inform user with suggested action.
    #[error("Bracket {0} has started{1}")]
    Started(BracketId, String),
    /// Bracket has not started. Inform user with suggested action.
    #[error("Bracket {0} has not started{1}")]
    NotStarted(BracketId, String),
    /// Error progressing the bracket
    #[error("{1}\nBracket: {0}")]
    Progression(BracketId, ProgressError),
}

/// Bracket identifier
pub type Id = Uuid;

/// Active brackets
pub type ActiveBrackets = HashMap<DiscussionChannelId, Id>;

/// Finalized brackets
pub type FinalizedBrackets = HashSet<Id>;

/// Add/remove players to the bracket. Then use methods to make the bracket
/// progress.
///
/// Implementation details is delegated to `Progression` struct.
///
/// Seeding is important: <https://youtu.be/ZGoIIV55hEc?t=108>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bracket {
    /// Identifier of this bracket
    bracket_id: Id,
    /// Name of this bracket
    bracket_name: String,
    /// Players of this bracket
    participants: Participants,
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

impl Bracket {
    /// Create new bracket
    #[must_use]
    pub fn new(
        name: &str,
        format: Format,
        seeding_method: SeedingMethod,
        start_time: DateTime<Utc>,
        automatic_match_validation: bool,
    ) -> Self {
        Self {
            bracket_id: BracketId::new_v4(),
            bracket_name: name.to_string(),
            participants: Participants::default(),
            matches: vec![],
            format,
            seeding_method,
            start_time,
            accept_match_results: false,
            automatic_match_progression: automatic_match_validation,
            is_closed: false,
        }
    }

    /// Report result for a match in this bracket. Returns updated bracket,
    /// match id where result is reported and new generated matches if
    /// automatic match validation is on.
    ///
    /// # Errors
    /// thrown when result cannot be parsed
    pub fn report_result(
        self,
        player_id: PlayerId,
        result: (i8, i8),
    ) -> Result<(Bracket, MatchId, Vec<Match>), Error> {
        if !self.accept_match_results {
            return Err(Error::NotStarted(
                self.bracket_id,
                ". Match results are not yet accepted".into(),
            ));
        }
        let p = self.format.get_progression(
            self.matches.clone(),
            self.participants.clone(),
            self.automatic_match_progression,
        );
        let (matches, affected_match_id, new_matches) = match p.report_result(player_id, result) {
            Ok(el) => el,
            Err(e) => return Err(Error::Progression(self.bracket_id, e)),
        };
        Ok((Self { matches, ..self }, affected_match_id, new_matches))
    }

    /// Report results for player 1 and the reverse result for the other
    /// player. Returns updated bracket, affected match id and new matches
    ///
    /// Assuming physically, both players comes up to the tournament organiser
    /// to report the result, then both player agree on the match outcome.
    ///
    /// # Errors
    /// thrown when result cannot be parsed
    pub fn tournament_organiser_reports_result(
        self,
        player1: PlayerId,
        result_player1: (i8, i8),
        player2: PlayerId,
    ) -> Result<(Bracket, MatchId, Vec<Match>), Error> {
        let p = self.format.get_progression(
            self.get_matches(),
            self.get_participants(),
            self.automatic_match_progression,
        );
        let (matches, affected_match_id, new_matches) =
            match p.tournament_organiser_reports_result(player1, result_player1, player2) {
                Ok(el) => el,
                Err(e) => return Err(Error::Progression(self.bracket_id, e)),
            };
        Ok((Self { matches, ..self }, affected_match_id, new_matches))
    }

    /// Start bracket: bar people from entering and accept match results.
    /// Returns updated bracket and matches to play
    ///
    /// # Errors
    /// thrown if there is not enough participants
    pub fn start(self) -> Result<(Self, Vec<Match>), Error> {
        if self.matches.is_empty() {
            return Err(Error::NotStarted(
                self.bracket_id,
                ". Matches need to be generated yet".into(),
            ));
        }
        let matches = self.matches_to_play();
        Ok((
            Self {
                is_closed: true,
                accept_match_results: true,
                ..self
            },
            matches,
        ))
    }

    /// Returns all matches that can be played out
    #[must_use]
    pub fn matches_to_play(&self) -> Vec<Match> {
        self.format
            .get_progression(
                self.get_matches(),
                self.get_participants(),
                self.automatic_match_progression,
            )
            .matches_to_play()
    }
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
        write!(f, "{} ({})", self.bracket_name, self.bracket_id)
    }
}

/// Parameters to create a bracket
pub struct CreateRequest<'b> {
    /// Automatically validate match if both players agree
    pub automatic_match_validation: bool,
    /// requested bracket format
    pub bracket_format: &'b str,
    /// requested bracket name
    pub bracket_name: &'b str,
    /// Id of internal channel
    pub internal_channel_id: &'b str,
    /// Organiser id of requested bracket while using service
    pub internal_organiser_id: &'b str,
    /// Organiser name of requested bracket
    pub organiser_name: &'b str,
    /// seeding method of requested bracket
    pub seeding_method: &'b str,
    /// Type of service used to make request
    pub service: &'b str,
    /// Advertised start time
    pub start_time: &'b str,
}

impl TryFrom<CreateRequest<'_>> for Bracket {
    type Error = ParsingError;

    fn try_from(br: CreateRequest) -> Result<Self, Self::Error> {
        Ok(Bracket::new(
            br.bracket_name,
            br.bracket_format.parse()?,
            br.seeding_method.parse()?,
            br.start_time.parse()?,
            br.automatic_match_validation,
        ))
    }
}

impl Default for Bracket {
    fn default() -> Self {
        Bracket::new(
            "new bracket",
            Format::default(),
            SeedingMethod::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            true,
        )
    }
}
