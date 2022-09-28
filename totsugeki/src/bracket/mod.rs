//! Bracket domain

mod disqualification;
mod getter_setter;
pub mod http_responses;
mod participants;
mod progression;
mod query_state;
pub mod raw;
mod seeding;

use crate::{
    bracket::Id as BracketId,
    format::{Format, ParsingError as FormatParsingError},
    matches::{Error as MatchError, Id as MatchId, Match, MatchParsingError, ReportedResult},
    opponent::Opponent,
    player::{Error as PlayerError, Id as PlayerId, Participants, Player},
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
#[derive(Error, Debug)]
pub enum Error {
    /// Error while seeding a bracket
    #[error("{0}")]
    Seeding(#[from] SeedingError),
    /// Error while updating players of bracket
    #[error("{0}")]
    PlayerUpdate(#[from] PlayerError),
    /// Match referred does not exist for this bracket
    #[error("Match with id \"{0}\" does not exists in bracket")]
    UnknownMatch(MatchId),
    /// Match cannot be updated
    #[error("{0}")]
    Match(#[from] MatchError),
    /// Unknown player provided for seeding
    #[error("Unknown player \"{0}\" cannot be used for seeding. Use the following players: {1} of bracket {2}")]
    UnknownPlayer(PlayerId, Participants, BracketId),
    /// Cannot add player when they are barred from entering
    #[error("Bracket \"{1}\" does not accept new participants")]
    BarredFromEntering(PlayerId, BracketId),
    /// Bracket does not accept result at the moment
    ///
    /// This happens when bracket has not started yet or has ended
    #[error("Bracket \"{1}\" does not accept reported results at the moment")]
    AcceptResults(PlayerId, BracketId),
    /// Player reported a result but there is no match for him to play
    #[error("There is no match to update in bracket \"{1}\"")]
    NoMatchToPlay(Player, BracketId),
    /// No matches where generated for this bracket
    #[error("No matches were generated yet for bracket {0}")]
    NoGeneratedMatches(BracketId),
    /// There is no match to play because player won the bracket
    #[error("There is no match for you to play because you won the bracket {1}")]
    NoNextMatch(PlayerId, BracketId),
    /// There is no match to play for player querying because he was eliminated
    /// from the bracket
    #[error("There is no match for you to play because you were eliminated from bracket {1}")]
    EliminatedFromBracket(PlayerId, BracketId),
    /// Only player in bracket can query for their next opponent
    #[error(
        "You do not have next opponent because you are not a participant of this bracket (\"{1}\")"
    )]
    PlayerIsNotParticipant(PlayerId, Id),
    /// Bracket has started. Inform user with suggested action.
    #[error("Bracket {0} has started{1}")]
    Started(BracketId, String),
    /// Bracket has not started. Inform user with suggested action.
    #[error("Bracket {0} has not started{1}")]
    NotStarted(BracketId, String),
    /// Bracket is over: all matches were played
    #[error("Bracket {0} is over")]
    AllMatchesPlayed(BracketId),
    /// Player is disqualified, no update permitted
    #[error("Player {1} is disqualified from bracket {0}")]
    PlayerDisqualified(BracketId, PlayerId),
    /// Player query is impossible because they are disqualified
    #[error("You are disqualified (DQ'ed) from bracket {0}")]
    DisqualifiedPlayerHasNoNextOpponent(BracketId, PlayerId),
    /// There is no match to update with given id
    #[error("There is no match to update with id {1}")]
    NoMatchToUpdate(Vec<Match>, MatchId),
}

/// Bracket identifier
pub type Id = Uuid;

/// Active brackets
pub type ActiveBrackets = HashMap<DiscussionChannelId, Id>;

/// Finalized brackets
pub type FinalizedBrackets = HashSet<Id>;

/// Bracket for tournament
///
/// Seeding is important: <https://youtu.be/ZGoIIV55hEc?t=108>
///
/// TLDR: do not match good players against each other early on so good players
/// don't end up below players they would have beaten otherwise
// TODO add factor to not match local players against each other
// idea: 1st and 2nd player get placed separately, then try to avoid matching
// any two players from the same local for the first round at least. What you
// would really want is to put players from the same local as far away from
// each of them as possible
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
    automatic_match_validation: bool,
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
            automatic_match_validation,
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
        let result = ReportedResult(result);
        if !self.accept_match_results {
            return Err(Error::AcceptResults(player_id, self.bracket_id));
        }
        if self.is_disqualified(player_id) {
            return Err(Error::PlayerDisqualified(self.bracket_id, player_id));
        }
        let old_matches = self.matches_to_play();
        let match_to_update = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let participants = self.get_participants().get_players_list();
        let player = participants
            .iter()
            .find(|p| p.get_id() == player_id)
            .expect("player");
        let bracket_id = self.get_id();
        match match_to_update {
            Some(m) => {
                let affected_match_id = m.get_id();
                let bracket = self.update_match_result(affected_match_id, result, player_id)?;

                let bracket = if bracket.automatic_match_validation {
                    match bracket.clone().validate_match_result(affected_match_id) {
                        Ok((b, _)) => b,
                        Err(e) => match e {
                            Error::Match(MatchError::PlayersReportedDifferentMatchOutcome(_)) => {
                                bracket
                            }
                            _ => return Err(e),
                        },
                    }
                } else {
                    bracket
                };

                let matches = bracket
                    .matches_to_play()
                    .iter()
                    .filter(|m| !old_matches.iter().any(|old_m| old_m.get_id() == m.get_id()))
                    .map(std::clone::Clone::clone)
                    .collect();
                Ok((bracket, affected_match_id, matches))
            }
            None => Err(Error::NoMatchToPlay(player.clone(), bracket_id)),
        }
    }

    /// Clear reported result from player and return updated bracket.
    ///
    /// Used before tournament organisers report to prevent an unnecessary
    /// update
    fn clear_reported_result(self, player_id: PlayerId) -> Result<Self, Error> {
        let match_to_update = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let participants = self.get_participants().get_players_list();
        let player = participants
            .iter()
            .find(|p| p.get_id() == player_id)
            .expect("player");
        match match_to_update {
            Some(m_to_clear) => {
                let m_to_clear = m_to_clear.clone().clear_reported_result(player_id);

                let matches = self
                    .matches
                    .into_iter()
                    .map(|m| {
                        if m.get_id() == m_to_clear.get_id() {
                            m_to_clear.clone()
                        } else {
                            m
                        }
                    })
                    .collect();
                Ok(Bracket { matches, ..self })
            }
            None => Err(Error::NoMatchToPlay(player.clone(), self.bracket_id)),
        }
    }

    /// Report results for player 1 and the reverse result for the other
    /// player. Returns updated bracket, affected match id and new matches
    ///
    /// Assuming physically, both players comes up to the tournament organiser
    /// to report the result, then both player agree on the match outcome.
    ///
    /// # Errors
    /// thrown when result cannot be parsed
    ///
    /// # Panics
    /// When both affected matches are not the same
    pub fn tournament_organiser_reports_result(
        self,
        player_1: PlayerId,
        result_player_1: (i8, i8),
        player_2: PlayerId,
    ) -> Result<(Bracket, MatchId, Vec<Match>), Error> {
        let result_player_1 = ReportedResult(result_player_1);
        let bracket = self.clear_reported_result(player_1)?;
        let bracket = bracket.clear_reported_result(player_2)?;
        let (bracket, first_affected_match, _new_matches) =
            bracket.report_result(player_1, result_player_1.0)?;
        let (bracket, second_affected_match, new_matches_2) =
            bracket.report_result(player_2, result_player_1.reverse().0)?;
        assert_eq!(first_affected_match, second_affected_match);
        Ok((bracket, first_affected_match, new_matches_2))
    }

    /// Start bracket: bar people from entering and accept match results.
    /// Returns updated bracket and matches to play
    ///
    /// # Errors
    /// thrown if there is not enough participants
    pub fn start(self) -> Result<(Self, Vec<Match>), Error> {
        if self.matches.is_empty() {
            return Err(Error::NoGeneratedMatches(self.bracket_id));
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

    /// Report match result and returns updated bracket. This does not update
    /// other matches.
    ///
    /// # Errors
    /// Thrown when given match id does not correspond to any match in the bracket
    fn update_match_result(
        self,
        match_id: MatchId,
        result: ReportedResult,
        player_id: PlayerId,
    ) -> Result<Bracket, Error> {
        let m = match self.matches.iter().find(|m| m.get_id() == match_id) {
            Some(m) => m,
            None => return Err(Error::UnknownMatch(match_id)),
        };

        let updated_match = m.clone().update_reported_result(player_id, result)?;
        let matches = self
            .matches
            .clone()
            .iter()
            .map(|m| {
                if m.get_id() == updated_match.get_id() {
                    updated_match.clone()
                } else {
                    m.clone()
                }
            })
            .collect();
        Ok(Self { matches, ..self })
    }

    /// Returns all matches that can be played out
    #[must_use]
    pub fn matches_to_play(&self) -> Vec<Match> {
        self.matches
            .clone()
            .into_iter()
            .filter(Match::is_playable)
            .collect()
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
