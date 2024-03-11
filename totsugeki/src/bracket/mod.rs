//! Bracket domain

mod assertions;
pub mod builder;
mod disqualification;
pub mod double_elimination_variant;
mod getter_setter;
pub mod matches;
mod ongoing;
mod participants;
mod progression;
mod query_state;
mod seeding;
pub mod single_elimination_variant;
mod winner_bracket;

use crate::{
    bracket::{matches::Error as ProgressError, Id as BracketId},
    format::{Format, ParsingError as FormatParsingError},
    matches::{Error as MatchError, Id as MatchId, Match, MatchParsingError},
    player::{Error as PlayerError, Id as PlayerId, Participants, Player},
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
            id: BracketId::new_v4(),
            name: name.to_string(),
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

    /// Update name of bracket
    #[must_use]
    pub fn update_name(self, name: String) -> Bracket {
        Self { name, ..self }
    }
    /// Add participant to bracket
    ///
    /// # Errors
    /// when bracket has started
    pub fn add_participant(&self, name: &str) -> Result<Bracket, Error> {
        if self.accept_match_results {
            return Err(Error::Started(
                self.id,
                "Bracket has started. You may not enter at this time.".into(),
            ));
        }
        let participants = self
            .participants
            .clone()
            .add_participant(Player::new(name.into()))?;
        let bracket = self.clone().regenerate_matches(participants)?;
        Ok(bracket)
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
                self.id,
                ". Match results are not yet accepted".into(),
            ));
        }
        let p = self.format.get_progression(
            self.matches.clone(),
            &self.participants,
            self.automatic_match_progression,
        );
        let (matches, affected_match_id, new_matches) = match p.report_result(player_id, result) {
            Ok(el) => el,
            Err(e) => return Err(self.get_from_progression_error(e)),
        };
        let bracket = Self { matches, ..self };
        bracket.check_all_assertions();
        Ok((bracket, affected_match_id, new_matches))
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
            &self.get_participants(),
            self.automatic_match_progression,
        );
        let (matches, affected_match_id, new_matches) =
            match p.tournament_organiser_reports_result(player1, result_player1, player2) {
                Ok(el) => el,
                Err(e) => return Err(self.get_from_progression_error(e)),
            };
        let bracket = Self { matches, ..self };
        bracket.check_all_assertions();
        Ok((bracket, affected_match_id, new_matches))
    }

    /// Start bracket: bar people from entering and accept match results.
    /// Returns updated bracket and matches to play
    ///
    /// # Errors
    /// thrown if there is not enough participants
    pub fn start(self) -> Result<(Self, Vec<Match>), Error> {
        if self.matches.is_empty() {
            return Err(Error::NotStarted(
                self.id,
                ". Matches need to be generated yet".into(),
            ));
        }
        let matches = self.matches_to_play();
        let bracket = Self {
            is_closed: true,
            accept_match_results: true,
            ..self
        };
        bracket.check_all_assertions();
        Ok((bracket, matches))
    }

    /// Returns all matches that can be played out
    #[must_use]
    pub fn matches_to_play(&self) -> Vec<Match> {
        self.format
            .get_progression(
                self.get_matches(),
                &self.get_participants(),
                self.automatic_match_progression,
            )
            .matches_to_play()
    }

    /// Summarise bracket state
    #[must_use]
    pub fn summary(&self) -> String {
        let mut r = self.name.to_string();
        for p in self.participants.get_players_list() {
            r = format!("{r}\n\t* {}", p.get_name());
        }
        for m in self.get_matches() {
            r = format!("{r}\n\t* {}", m.summary_with_name(&self.get_participants()));
        }
        r
    }

    /// Add bracket id to error message and maps player name from player id
    // NOTE: I hope there will be a better way to add additionnal info to an
    // error because this does not scale over time
    pub(crate) fn get_from_progression_error(&self, pe: ProgressError) -> Error {
        match pe {
            ProgressError::Disqualified(player_id) => Error::Disqualified(
                self.id,
                self.participants
                    .get(player_id)
                    .expect("disqualified player id"),
            ),
            ProgressError::NoNextMatch(player_id) => Error::NoNextMatch(
                self.id,
                self.participants
                    .get(player_id)
                    .expect("player id with no next match"),
            ),
            ProgressError::Eliminated(player_id) => Error::Eliminated(
                self.id,
                self.participants
                    .get(player_id)
                    .expect("eliminated player id"),
            ),
            ProgressError::PlayerIsNotParticipant(player_id) => Error::PlayerIsNotParticipant(
                self.id,
                self.participants
                    .get(player_id)
                    .expect("player id of non-participant"),
            ),
            ProgressError::ForbiddenDisqualified(player_id) => Error::ForbiddenDisqualified(
                self.id,
                self.participants
                    .get(player_id)
                    .expect("disqualified player id for which bracket update cannot be performed"),
            ),
            ProgressError::Seeding(e) => Error::Seeding(e),
            ProgressError::NoGeneratedMatches => Error::NoGeneratedMatches(self.id),
            ProgressError::TournamentIsOver => Error::TournamentIsOver(self.id),
            ProgressError::MatchUpdate(me) => Error::MatchUpdate(self.id, me),
            ProgressError::UnknownPlayer(player_id, _players) => {
                Error::UnknownPlayer(player_id, self.participants.clone(), self.id)
            }
            ProgressError::NoMatchToPlay(player_id) => Error::NoMatchToPlay(
                self.id,
                self.participants
                    .get(player_id)
                    .expect("disqualified player id for which bracket update cannot be performed"),
            ),
            ProgressError::UnknownMatch(match_id) => Error::UnknownMatch(self.id, match_id),
            ProgressError::NoMatchToUpdate(matches, m) => {
                Error::NoMatchToUpdate(self.id, matches, m)
            }
        }
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
        write!(f, "{} ({})", self.name, self.id)
    }
}

impl Default for Bracket {
    fn default() -> Self {
        Bracket::new(
            "",
            Format::default(),
            SeedingMethod::default(),
            Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
            true,
        )
    }
}

/// Errors while manipulating bracket
#[derive(Debug)]
pub enum PartitionError {
    /// You need at least 3 players to perform this operation
    NotEnoughPlayersInBracket,
}
