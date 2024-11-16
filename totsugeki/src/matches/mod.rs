//! Two players play a match, resulting in a winner and a loser

mod bracket_result;
mod query;

use crate::player::PlayerID;
use crate::{
    opponent::{Opponent, ParsingOpponentError},
    player::Participants,
    ID,
};
pub use bracket_result::{BracketResult, Error as BracketResultGenerationError};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::{num::ParseIntError, str::FromStr};
use thiserror::Error;

/// Match ID
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, Copy)]
pub struct MatchID(ID);

impl FromStr for MatchID {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = ID::parse_str(s)?;
        Ok(MatchID(id))
    }
}

impl MatchID {
    /// Match ID
    #[must_use]
    pub fn new() -> Self {
        Self(ID::new_v4())
    }
}

impl Display for MatchID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Match ID {}", self.0)
    }
}

/// Error while interacting with match
#[derive(Error, Debug, Clone)]
pub enum Error {
    /// Can only update match when both player reported
    #[error("Missing report: {:?} and {:?} were reported", .1[0], .1[1])]
    MissingReport(MatchID, MatchReportedResult),
    /// Players reported different match outcome
    #[error("Players reported different match outcomes: {} and {} were reported", .1[0], .1[1])]
    PlayersReportedDifferentMatchOutcome(MatchID, [ReportedResult; 2]),
    /// Mathematical overflow happened, cannot proceed
    #[error("Error. Unable to proceed further.")]
    MathOverflow,
    /// Cannot update match because player is Unknown
    #[error("Player with id \"{0}\" is unknown. Players in this match are: {} VS {}", .1[0], .1[1])]
    UnknownPlayer(PlayerID, MatchPlayers),
    /// Cannot update match result because an opponent is missing
    #[error("Cannot report result in a match where opponent is missing. Current players: {} VS {}", .0[0], .0[1])]
    MissingOpponent(MatchPlayers),
    /// Cannot instantiate match with two same player
    #[error("Error. Cannot use same player as both player of a match.")]
    SamePlayer,
    /// Cannot set opponent without a player id
    #[error("Need a player id for opponent")]
    OpponentIsNotAPlayer,
    /// No opponent to player was found
    #[error("Incomplete match")]
    NoOpponent(MatchPlayers),
}

/// Match generation error
#[derive(Error, Debug)]
pub enum GenerationError {
    /// Cannot instantiate match with two same player
    #[error("Error. Cannot use same player as both player of a match.")]
    SamePlayer,
}

impl From<GenerationError> for Error {
    fn from(_value: GenerationError) -> Self {
        Self::SamePlayer
    }
}

/// Seeds of players
pub type Seeds = [usize; 2];

/// A match result is a score. For example, I win 2-0
pub type MatchReportedResult = [Option<(i8, i8)>; 2];

/// Reported result, where first number is player own score, and second score is their opponent
/// score (example: I won 2-0, or I lost 0-2)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReportedResult(pub Option<(i8, i8)>);

impl PartialEq<ReportedResult> for ReportedResult {
    fn eq(&self, other: &ReportedResult) -> bool {
        match (self, *other) {
            (ReportedResult(Some((s11, s12))), ReportedResult(Some((s21, s22)))) => {
                *s11 == s21 && *s12 == s22
            }
            _ => false,
        }
    }
}

impl ReportedResult {
    /// Reverse score
    #[must_use]
    pub fn reverse(self) -> Self {
        match self.0 {
            Some((s1, s2)) => ReportedResult(Some((s2, s1))),
            None => ReportedResult(None),
        }
    }
}

impl std::fmt::Display for ReportedResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportedResult(Some((s1, s2))) => write!(f, "{s1}-{s2}"),
            ReportedResult(None) => write!(f, "?"),
        }
    }
}

/// Error while parsing match result
#[derive(Error, Debug)]
pub enum MatchResultParsingError {
    /// Could not parse
    #[error("{0} does not respect result format. Please report result as 'X-Y'")]
    MissingResultDelimiter(String),
    /// Could not parse integer
    #[error("Could not parse integer")]
    Result(#[from] ParseIntError),
}

impl FromStr for ReportedResult {
    type Err = MatchResultParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('-') {
            Some((l, r)) => {
                let l_score: i8 = l.parse::<i8>()?;
                let r_score: i8 = r.parse::<i8>()?;
                Ok(Self(Some((l_score, r_score))))
            }
            None => Err(MatchResultParsingError::MissingResultDelimiter(s.into())),
        }
    }
}

/// Players in match
pub type MatchPlayers = [Opponent; 2];

/// A match between two players, resulting in a winner and a loser
///
/// When updating a bracket, the match state can affect behaviour:
/// * expected seeding (example: grand finals should be between seed 1 and 2)
/// * winner of match (who moves on)
/// * player is disqualified for the match
/// * player reported results for this match (especially when reports differ)
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, Copy)]
pub struct Match {
    /// Identifier of match
    pub(crate) id: MatchID,
    /// Participants
    pub(crate) players: MatchPlayers,
    /// seeds\[0\]: top seed
    /// seeds\[1\]: bottom seed
    pub(crate) seeds: Seeds,
    /// The winner of this match
    pub(crate) winner: Opponent,
    /// The loser of this match by disqualification
    pub(crate) automatic_loser: Opponent,
    /// Result reported by players
    pub(crate) reported_results: MatchReportedResult,
}

impl Display for Match {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\t{} vs {}", self.players[0], self.players[1])?;
        writeln!(f, "winner: {}", self.winner)
    }
}

impl Match {
    /// Player on left side has the higher expected seed
    #[must_use]
    pub fn display_player_high_seed(&self) -> String {
        match self.players[0] {
            Opponent(Some(p)) => p.to_string(),
            Opponent(None) => "?".into(),
        }
    }
    /// Player on right side has the lower expected seed
    #[must_use]
    pub fn display_player_low_seed(&self) -> String {
        match self.players[1] {
            Opponent(Some(p)) => p.to_string(),
            Opponent(None) => "?".into(),
        }
    }
}

impl Match {
    /// Summary of match
    #[must_use]
    pub fn summary(&self) -> String {
        let p1 = &self.display_player_high_seed();
        let p2 = &self.display_player_low_seed();

        let p1_status = match (self.winner, self.automatic_loser, self.players[0]) {
            (Opponent(Some(w)), _, Opponent(Some(p1))) if w == p1 => "W",
            (_, Opponent(Some(l)), Opponent(Some(p1))) if l == p1 => "L",
            _ => "-",
        };
        let p2_status = match (self.winner, self.automatic_loser, self.players[1]) {
            (Opponent(Some(w)), _, Opponent(Some(p2))) if w == p2 => "W",
            (_, Opponent(Some(l)), Opponent(Some(p2))) if l == p2 => "L",
            _ => "-",
        };
        format!(
            "{:?} {p1_status}{p1:02} VS {p2_status}{p2:02} | match id: {}",
            self.seeds, self.id
        )
    }

    /// Summary of match
    ///
    /// # Panics
    /// when player in match is not among the provided participants
    #[must_use]
    pub fn summary_with_name(&self, participants: &Participants) -> String {
        // TODO nitpick could be nicer, results of 3-4 refactoring of types...
        let p1 = match self.players[0] {
            Opponent(Some(id)) => participants.get(id).expect("player").get_name(),
            Opponent(None) => Opponent(None).to_string(),
        };
        let p2 = match self.players[1] {
            Opponent(Some(id)) => participants.get(id).expect("player").get_name(),
            Opponent(None) => Opponent(None).to_string(),
        };

        let p1_status = match (self.winner, self.automatic_loser, self.players[0]) {
            (Opponent(Some(w)), _, Opponent(Some(p1))) if w == p1 => "W",
            (_, Opponent(Some(l)), Opponent(Some(p1))) if l == p1 => "L",
            _ => "-",
        };
        let p2_status = match (self.winner, self.automatic_loser, self.players[1]) {
            (Opponent(Some(w)), _, Opponent(Some(p2))) if w == p2 => "W",
            (_, Opponent(Some(l)), Opponent(Some(p2))) if l == p2 => "L",
            _ => "-",
        };
        format!(
            "{:?} {p1_status}{p1:02} VS {p2_status}{p2:02} | match id: {}",
            self.seeds, self.id
        )
    }

    /// Returns `true` when all players involved in match have reported a
    /// result, `false` otherwise
    #[must_use]
    pub fn has_all_player_reports(&self) -> bool {
        matches!(self.reported_results, [Some(_r1), Some(_r2)])
    }

    /// Returns `true` when both players are present, `false` otherwise
    #[must_use]
    pub fn both_opponents_are_present(&self) -> bool {
        matches!(self.players, [Opponent(Some(_)), Opponent(Some(_))])
    }
}

/// Compose double elimination matches from partition
pub(crate) fn double_elimination_matches_from_partition(
    winners: &[Match],
    losers: &[Match],
    grand_finals: Match,
    reset: Match,
) -> Vec<Match> {
    let mut matches: Vec<Match> = winners.into();
    matches.append(&mut losers.into());
    matches.push(grand_finals);
    matches.push(reset);
    matches
}

impl Match {
    /// Clear result from match and returns updated match
    #[must_use]
    pub(crate) fn clear_reported_result(self, player_id: PlayerID) -> Self {
        assert!(
            self.contains(player_id),
            "cannot clear result of match for unknown player"
        );
        match self.players {
            [Opponent(Some(p1)), _] if p1 == player_id => Self {
                reported_results: [None, self.reported_results[1]],
                ..self
            },
            [_, Opponent(Some(p2))] if p2 == player_id => Self {
                reported_results: [self.reported_results[0], None],
                ..self
            },
            _ => unreachable!("cannot clear result for unknown player"),
        }
    }

    /// Returns true if one of the player has id `player_id`
    #[must_use]
    pub fn contains(&self, player_id: PlayerID) -> bool {
        match self.players {
            [Opponent(Some(p1)), _] if p1 == player_id => true,
            [_, Opponent(Some(p2))] if p2 == player_id => true,
            _ => false,
        }
    }

    /// Get id of match
    #[must_use]
    pub fn get_id(&self) -> MatchID {
        self.id
    }

    /// Get automatic looser of match. Loosers are always players
    #[must_use]
    pub fn get_automatic_loser(&self) -> Opponent {
        self.automatic_loser
    }

    /// Get players for this match
    #[must_use]
    pub fn get_players(&self) -> MatchPlayers {
        self.players
    }

    /// Get seeds of (predicted) player
    #[must_use]
    pub fn get_seeds(&self) -> Seeds {
        self.seeds
    }

    /// Get winner of match. Winners are players
    #[must_use]
    pub fn get_winner(&self) -> Opponent {
        self.winner
    }

    /// Returns true if player the automatic looser of this match is given
    /// player
    #[must_use]
    pub(crate) fn is_automatic_loser_by_disqualification(&self, player_id: PlayerID) -> bool {
        matches!(self.automatic_loser, Opponent(Some(loser)) if loser == player_id)
    }

    #[must_use]
    /// Returns true if this match is where loser arrives
    pub(crate) fn is_first_loser_match(&self, expected_seed: usize) -> bool {
        self.seeds[0] == expected_seed || self.seeds[1] == expected_seed
    }

    /// Returns true if match is over
    #[must_use]
    pub fn is_over(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self.players, self.winner, self.automatic_loser) {
            ([Opponent(Some(_p1)), Opponent(Some(_p2))], Opponent(Some(_winner)), _) => true,
            ([Opponent(Some(_p1)), Opponent(Some(_p2))], _, Opponent(Some(_winner))) => true,
            _ => false,
        }
    }

    /// Returns true if both opponents are present but a winner has yet to be
    /// declared. Returns false if it can be resolved automatically (because of
    /// a disqualification)
    #[must_use]
    pub fn needs_playing(&self) -> bool {
        self.winner == Opponent(None)
            && self.automatic_loser == Opponent(None)
            && self.players[0] != Opponent(None)
            && self.players[1] != Opponent(None)
    }

    /// Create looser bracket match where opponents are unknown yet
    #[must_use]
    #[cfg(test)]
    pub(crate) fn looser_bracket_match(id: MatchID, seeds: [usize; 2]) -> Self {
        Match {
            id,
            players: [Opponent(None), Opponent(None)],
            seeds,
            winner: Opponent(None),
            automatic_loser: Opponent(None),
            reported_results: [None, None],
        }
    }

    /// Returns true if participant of match is disqualified but winner is not
    /// declared
    #[must_use]
    pub fn needs_update_because_of_disqualified_participant(&self) -> bool {
        self.winner == Opponent(None)
            && self.automatic_loser != Opponent(None)
            && self.players[0] != Opponent(None)
            && self.players[1] != Opponent(None)
    }

    /// Create new match with two opponents
    ///
    /// Winner is automatically set if bye opponent is set
    ///
    /// # Errors
    /// Returns an error if both players are the same (two unknown players will
    /// not produce an error)
    pub fn new(players: [Opponent; 2], seeds: [usize; 2]) -> Result<Match, GenerationError> {
        match players {
            [Opponent(Some(p1)), Opponent(Some(p2))] if p1 == p2 => {
                Err(GenerationError::SamePlayer)
            }
            _ => Ok(Self {
                id: MatchID::new(),
                players,
                winner: Opponent(None),
                automatic_loser: Opponent(None),
                seeds,
                reported_results: [None, None],
            }),
        }
    }

    /// Create new match with two opponents
    ///
    /// Winner is automatically set if bye opponent is set
    ///
    /// # Errors
    /// Returns an error if both players are the same (two unknown players will
    /// not produce an error)
    #[must_use]
    pub fn new_empty(seeds: [usize; 2]) -> Match {
        Self {
            id: MatchID::new(),
            players: [Opponent(None), Opponent(None)],
            winner: Opponent(None),
            automatic_loser: Opponent(None),
            seeds,
            reported_results: [None, None],
        }
    }

    /// Create new looser bracket match where opponents are unknown yet
    #[must_use]
    pub fn new_looser_bracket_match(seeds: [usize; 2]) -> Self {
        Match {
            id: MatchID::new(),
            players: [Opponent(None), Opponent(None)],
            seeds,
            winner: Opponent(None),
            automatic_loser: Opponent(None),
            reported_results: [None, None],
        }
    }

    /// Set looser of this match (when disqualified)
    ///
    /// # Panics
    /// * looser is not a participant of the match
    pub fn set_automatic_loser_(&mut self, player_id: PlayerID) {
        assert!(self.contains(player_id), "player {player_id} in match");

        let loser = match self.players {
            [Opponent(Some(p1)), _] if p1 == player_id => self.players[0],
            [_, Opponent(Some(p2))] if p2 == player_id => self.players[1],
            _ => Opponent(None),
        };

        self.automatic_loser = loser;
    }
    /// Set looser of this match (when disqualified)
    ///
    /// # Errors
    /// thrown when looser is not a participant of the match
    /// # Panics
    /// When player does not belong to the match
    #[must_use]
    pub fn set_automatic_loser(self, player_id: PlayerID) -> Self {
        assert!(self.contains(player_id), "player {player_id} in match");

        let loser = match self.players {
            [Opponent(Some(p1)), _] if p1 == player_id => self.players[0],
            [_, Opponent(Some(p2))] if p2 == player_id => self.players[1],
            _ => Opponent(None),
        };

        Self {
            automatic_loser: loser,
            ..self
        }
    }

    /// Set player of match
    ///
    /// Motivation for this function is to assert and fail rather than error
    /// out like in `insert_player`
    ///
    /// # Panics
    /// * if opponent is already present
    /// * someone was already set
    pub(crate) fn set_player(self, player_id: PlayerID, higher_seed: bool) -> Self {
        assert!(
            !self.contains(player_id),
            "cannot set opponent when already in the match"
        );
        let player = Opponent(Some(player_id));
        let players = if higher_seed {
            assert_eq!(self.players[0], Opponent(None));
            [player, self.players[1]]
        } else {
            assert_eq!(self.players[1], Opponent(None));
            [self.players[0], player]
        };
        Self { players, ..self }
    }

    /// Insert player in match and return updated match
    ///
    /// # Panics
    /// If match slot is not empty
    #[must_use]
    pub fn insert_player(self, player_id: PlayerID, is_player_1: bool) -> Match {
        match (is_player_1, self.players) {
            (true, [Opponent(Some(other_player)), _])
            | (false, [_, Opponent(Some(other_player))])
                if player_id != other_player =>
            {
                panic!("player {player_id} is already in match {}", self.id);
            }
            _ => {}
        }
        let player = Opponent(Some(player_id));
        let players = if is_player_1 {
            [player, self.players[1]]
        } else {
            [self.players[0], player]
        };
        Match { players, ..self }
    }

    /// Returns true if the stronger seed won. Returns None if winner cannot be
    /// determined.
    ///
    /// # Panics
    /// if seeds of players are equal
    #[must_use]
    pub(crate) fn stronger_seed_wins(&self) -> Option<bool> {
        assert_ne!(self.seeds[0], self.seeds[1]);
        let Opponent(Some(winner)) = self.winner else {
            return None;
        };

        match self.seeds {
            [s1, s2] if s1 < s2 => match self.players {
                [Opponent(Some(p1)), Opponent(Some(_))] if p1 == winner => Some(true),
                [Opponent(Some(_)), Opponent(Some(p2))] if p2 == winner => Some(false),
                _ => None,
            },
            [s1, s2] if s1 > s2 => match self.players {
                [Opponent(Some(p1)), Opponent(Some(_))] if p1 == winner => Some(false),
                [Opponent(Some(_)), Opponent(Some(p2))] if p2 == winner => Some(true),
                _ => None,
            },
            _ => unreachable!(),
        }
    }

    /// Set match outcome using reported results. Returns updated match, winner
    /// id (if possible) and looser id.
    ///
    /// When a player is disqualified (through `automatic_loser`), then outcome
    /// can be updated.
    ///
    /// # Errors
    /// Returns an error if reported scores don't agree on the winner
    /// # Panics
    /// When one of the players did not report match result
    pub(crate) fn update_outcome(self) -> Result<(Match, Option<PlayerID>, PlayerID), Error> {
        // if there is a disqualified player, try to set the winner
        if let Opponent(Some(dq_player)) = self.automatic_loser {
            return match self.players {
                [Opponent(Some(p1)), Opponent(Some(p2))] if p1 == dq_player => Ok((
                    Self {
                        winner: Opponent(Some(p2)),
                        ..self
                    },
                    Some(p2),
                    dq_player,
                )),
                [Opponent(Some(p1)), Opponent(Some(p2))] if p2 == dq_player => Ok((
                    Self {
                        winner: Opponent(Some(p1)),
                        ..self
                    },
                    Some(p1),
                    dq_player,
                )),
                [Opponent(Some(p1)), Opponent(None)] if p1 == dq_player => Ok((
                    Self {
                        winner: Opponent(None),
                        ..self
                    },
                    None,
                    dq_player,
                )),
                [Opponent(None), Opponent(Some(p2))] if p2 == dq_player => Ok((
                    Self {
                        winner: Opponent(None),
                        ..self
                    },
                    None,
                    dq_player,
                )),
                _ => unreachable!(),
            };
        }

        let same_result_reported = match self.reported_results {
            [Some((s11, s12)), Some((s21, s22))] => {
                ReportedResult(Some((s11, s12))).reverse() == ReportedResult(Some((s21, s22)))
            }
            _ => false,
        };

        let (winner, loser) = match (self.players, same_result_reported, self.reported_results) {
            #[allow(clippy::unnested_or_patterns)]
            ([Opponent(None), _], _, _) | ([_, Opponent(None)], _, _) => {
                return Err(Error::MissingOpponent(self.players));
            }
            (
                [Opponent(Some(p1)), Opponent(Some(p2))],
                true,
                [Some((s11, s12)), Some((s21, s22))],
            ) if s11 > s12 && s21 < s22 => (p1, p2),
            (
                [Opponent(Some(p1)), Opponent(Some(p2))],
                true,
                [Some((s11, s12)), Some((s21, s22))],
            ) if s11 < s12 && s21 > s22 => (p2, p1),
            (
                [Opponent(Some(_)), Opponent(Some(_))],
                false,
                [Some((s11, s12)), Some((s21, s22))],
            ) => {
                return Err(Error::PlayersReportedDifferentMatchOutcome(
                    self.id,
                    [
                        ReportedResult(Some((s11, s12))),
                        ReportedResult(Some((s21, s22))),
                    ],
                ));
            }
            #[allow(clippy::unnested_or_patterns)]
            ([Opponent(Some(_)), Opponent(Some(_))], _, [None, _])
            | ([Opponent(Some(_)), Opponent(Some(_))], _, [_, None]) => {
                return Err(Error::MissingReport(self.id, self.reported_results));
            }
            (_, _, r) => {
                unreachable!("{:?}", r)
            }
        };

        Ok((
            Match {
                id: self.id,
                players: self.players,
                seeds: self.seeds,
                winner: Opponent(Some(winner)),
                automatic_loser: self.automatic_loser,
                reported_results: self.reported_results,
            },
            Some(winner),
            loser,
        ))
    }

    /// Update match result and return updated match
    ///
    /// # Panics
    /// When referred player is not in the match or opponent has not been defined
    #[must_use]
    pub fn update_reported_result(self, player_id: PlayerID, result: ReportedResult) -> Self {
        match self.players {
            [Opponent(None), _] | [_, Opponent(None)] => {
                panic!("All opponents must be defined before reporting result")
            }
            [Opponent(Some(player1)), Opponent(Some(_))] if player1 == player_id => {
                let mut reported_results = self.reported_results;
                reported_results[0] = result.0;
                Match {
                    id: self.id,
                    players: self.players,
                    seeds: self.seeds,
                    winner: self.winner,
                    automatic_loser: self.automatic_loser,
                    reported_results,
                }
            }
            [Opponent(Some(_)), Opponent(Some(player2))] if player2 == player_id => {
                let mut reported_results = self.reported_results;
                reported_results[1] = result.0;
                Match {
                    id: self.id,
                    players: self.players,
                    seeds: self.seeds,
                    winner: self.winner,
                    automatic_loser: self.automatic_loser,
                    reported_results,
                }
            }
            _ => panic!("unknown player ID reported for match {}", self.id),
        }
    }

    /// Returns other player of this match
    ///
    /// # Errors
    /// thrown when there is no other player or player is not in the match
    pub fn get_other_player(&self, player_id: PlayerID) -> Result<PlayerID, Error> {
        match self.players {
            [Opponent(Some(p1)), Opponent(Some(p2))] if p1 == player_id => Ok(p2),
            [Opponent(Some(p1)), Opponent(Some(p2))] if p2 == player_id => Ok(p1),
            _ => Err(Error::NoOpponent(self.players)),
        }
    }

    /// Get score of match. Defaults to 0-0 if winner is not declared
    #[must_use]
    pub fn get_score(&self) -> Option<(i8, i8)> {
        match self.reported_results {
            [Some(r1), Some(r2)] if r1.0 == r2.1 && r1.1 == r2.0 => Some(r1),
            _ => None,
        }
    }
}

/// Error while converting response from network
#[derive(Error, Debug)]
pub enum MatchParsingError {
    /// Error while parsing opponent
    #[error("{0}")]
    ParsingOpponentError(#[from] ParsingOpponentError),
    /// Could not gather opponents for a match
    #[error(
        "Could not use opponents to generate match. Is there two opponents?\nOpponents: {0:?}"
    )]
    GatherOpponents(Vec<Opponent>),
    /// Reported result could not be parsed
    #[error("Reported results could not be parsed: {0}")]
    ReportedResult(#[from] MatchResultParsingError),
    /// Winner is not one of the players in the match
    #[error("Winner is not a participant of the match")]
    UnknownWinner,
    /// Looser is not one of the players in the match
    #[error("Looser is not a participant of the match")]
    UnknownLooser,
}

// NOTE: here because Vec<T> does not implement std::error::Error when used with try_into
impl From<Vec<Opponent>> for MatchParsingError {
    fn from(opponents: Vec<Opponent>) -> Self {
        MatchParsingError::GatherOpponents(opponents)
    }
}

/// Match identifier
// TODO look for possible way this could be interpreted as smth like Player ID
// and prevent that
pub type Id = uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Player, PlayerID};

    #[test]
    fn match_contains_both_players() {
        let p1 = PlayerID::create();
        let player_1 = Opponent(Some(p1));
        let p2 = PlayerID::create();
        let player_2 = Opponent(Some(p2));
        let unknown = PlayerID::create();
        let m = Match::new([player_1, player_2], [1, 2]).expect("match");
        assert!(m.contains(p1));
        assert!(m.contains(p2));
        assert!(!m.contains(unknown));
    }

    #[test]
    fn cannot_create_match_with_same_player() {
        let p = PlayerID::create();
        let player = Opponent(Some(p));
        match Match::new([player, player], [1, 2]) {
            Err(GenerationError::SamePlayer) => {}
            _ => panic!("Expected error but got none"),
        }
    }

    #[test]
    fn stronger_seeds_wins() {
        let p1 = Player::new("p1".into());
        let p2 = Player::new("p2".into());
        let m = Match::new(
            [Opponent(Some(p1.get_id())), Opponent(Some(p2.get_id()))],
            [1, 2],
        )
        .expect("match");
        assert!(!m.is_over());
        let m = m.update_reported_result(p1.get_id(), ReportedResult(Some((2, 0))));
        let m = m.update_reported_result(p2.get_id(), ReportedResult(Some((0, 2))));
        let (m, _, _) = m.update_outcome().expect("validation");
        assert!(m.is_over());
        assert!(
            m.stronger_seed_wins().expect("value"),
            "expected p1 with higher seed to win"
        );

        let m = Match::new(
            [Opponent(Some(p1.get_id())), Opponent(Some(p2.get_id()))],
            [1, 2],
        )
        .expect("match");
        assert!(!m.is_over());
        let m = m.update_reported_result(p2.get_id(), ReportedResult(Some((2, 0))));
        let m = m.update_reported_result(p1.get_id(), ReportedResult(Some((0, 2))));
        let (m, _, _) = m.update_outcome().expect("validation");
        assert!(m.is_over());
        assert!(
            !m.stronger_seed_wins().expect("value"),
            "expected p2 with lower seed to win"
        );

        let m = Match::new(
            [Opponent(Some(p1.get_id())), Opponent(Some(p2.get_id()))],
            [2, 1],
        )
        .expect("match");
        assert!(!m.is_over());
        let m = m.update_reported_result(p1.get_id(), ReportedResult(Some((2, 0))));
        let m = m.update_reported_result(p2.get_id(), ReportedResult(Some((0, 2))));
        let (m, _, _) = m.update_outcome().expect("validation");
        assert!(m.is_over());
        assert!(
            !m.stronger_seed_wins().expect("value"),
            "p1 with lower seed to win"
        );

        let m = Match::new(
            [Opponent(Some(p1.get_id())), Opponent(Some(p2.get_id()))],
            [2, 1],
        )
        .expect("match");
        assert!(!m.is_over());
        let m = m.update_reported_result(p2.get_id(), ReportedResult(Some((2, 0))));
        let m = m.update_reported_result(p1.get_id(), ReportedResult(Some((0, 2))));
        let (m, _, _) = m.update_outcome().expect("validation");
        assert!(m.is_over());
        assert!(
            m.stronger_seed_wins().expect("value"),
            "expected p2 with higher seed to win"
        );
    }

    #[test]
    fn playable_matches() {
        let p1 = Player::new("p1".into());
        let p2 = Player::new("p2".into());
        let m = Match::new(
            [Opponent(Some(p1.get_id())), Opponent(Some(p2.get_id()))],
            [1, 2],
        )
        .expect("match");
        assert!(m.needs_playing());

        let m = Match::new([Opponent(None), Opponent(Some(p2.get_id()))], [1, 2]).expect("match");
        assert!(!m.needs_playing());
        let m = Match::new([Opponent(Some(p1.get_id())), Opponent(None)], [1, 2]).expect("match");
        assert!(!m.needs_playing());
        let m = Match::new([Opponent(None), Opponent(None)], [1, 2]).expect("match");
        assert!(!m.needs_playing());
    }

    #[test]
    fn parse_results() {
        let to_test = vec![
            ("0-0", (0, 0)),
            ("4-0", (4, 0)),
            ("5-0", (5, 0)),
            ("0-4", (0, 4)),
            ("0-5", (0, 5)),
            // all intermediate score for FT2
            ("1-0", (1, 0)),
            ("2-0", (2, 0)),
            ("0-1", (0, 1)),
            ("0-2", (0, 2)),
            ("1-2", (1, 2)),
            ("2-1", (2, 1)),
            // all intermediate score for FT3
            ("3-0", (3, 0)),
            ("3-1", (3, 1)),
            ("3-2", (3, 2)),
            ("0-3", (0, 3)),
            ("1-3", (1, 3)),
            ("2-3", (2, 3)),
        ];
        for (s, (l_expected, r_expected)) in to_test {
            let ReportedResult(Some((l, r))) = s.parse::<ReportedResult>().expect("result") else {
                panic!("no result")
            };
            assert_eq!(
                l, l_expected,
                "could not parse {s} into ({l_expected}, {r_expected})"
            );
            assert_eq!(
                r, r_expected,
                "could not parse {s} into ({l_expected}, {r_expected})"
            );
        }
    }

    #[test]
    fn insert_players_in_empty_match() {
        let m = Match::default();
        let p1 = PlayerID::create();
        let p2 = PlayerID::create();
        let m = m.insert_player(p1, true);
        let _m = m.insert_player(p2, false);

        let m = Match::default();
        let m = m.insert_player(p2, false);
        let _m = m.insert_player(p1, true);
    }

    #[test]
    #[should_panic]
    fn cannot_insert_player_if_someone_else_is_already_there_p1_side() {
        let p1 = PlayerID::create();
        let p2 = PlayerID::create();
        let m = Match::new([Opponent(Some(p1)), Opponent(Some(p2))], [0, 0]).expect("match");
        let p1_intruder = PlayerID::create();

        let _ = m.insert_player(p1_intruder, true);
    }
    #[test]
    #[should_panic]
    fn cannot_insert_player_if_someone_else_is_already_there_p2_side() {
        let p1 = PlayerID::create();
        let p2 = PlayerID::create();
        let m = Match::new([Opponent(Some(p1)), Opponent(Some(p2))], [0, 0]).expect("match");

        let p2_intruder = PlayerID::create();
        let _ = m.insert_player(p2_intruder, false);
    }

    #[test]
    fn insert_player_even_if_already_present() {
        let p1 = PlayerID::create();
        let p2 = PlayerID::create();
        let m = Match::new([Opponent(Some(p1)), Opponent(Some(p2))], [0, 0]).expect("match");

        let m = m.insert_player(p1, true);
        let m = m.insert_player(p1, true);
        let m = m.insert_player(p1, true);

        let m = m.insert_player(p2, false);
        let m = m.insert_player(p2, false);
        let _m = m.insert_player(p2, false);
    }

    #[test]
    fn no_match_score_when_no_winner_is_declared() {
        let p1 = PlayerID::create();
        let p2 = PlayerID::create();
        let m = Match::new([Opponent(Some(p1)), Opponent(Some(p2))], [0, 0]).expect("match");

        assert!(m.get_score().is_none());
    }
}
