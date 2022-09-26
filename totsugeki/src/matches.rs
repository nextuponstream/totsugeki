//! Two players play a match, resulting in a winner and a loser

use crate::{
    bracket::Id as BracketId,
    opponent::{Opponent, ParsingOpponentError},
    player::{Id as PlayerId, Player},
};
#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

/// Error while interacting with match
#[derive(Error, Debug, Clone)]
pub enum Error {
    /// Players reported different match outcome
    #[error("Players reported different match outcomes: {} and {} were reported", .0[0], .0[1])]
    PlayersReportedDifferentMatchOutcome([ReportedResult; 2]),
    /// Mathematical overflow happened, cannot proceed
    #[error("Error. Unable to proceed further.")]
    MathOverflow,
    /// Cannot update match because player is Unknown
    #[error("Player with id \"{0}\" is unknown. Players in this match are: {} VS {}", .1[0], .1[1])]
    // FIXME change to player
    UnknownPlayer(PlayerId, MatchPlayers),
    /// Cannot update match result because an opponent is missing
    #[error("Cannot report result in a match where opponent is missing. Current players: {} VS {}", .0[0], .0[1])]
    MissingOpponent(MatchPlayers),
    /// Match got into a really bad state where an unknown player has result
    #[error("Error. Unable to proceed further.")]
    UnknownPlayerWithReportedResults,
    /// Cannot instantiate match with two same player
    #[error("Error. Cannot use same player as both player of a match.")]
    SamePlayer,
    /// Partitionning of double elimination matches cannot proceed
    #[error("Number of matches needs to be 2*n - 1, n being the number of participants, got: {0}")]
    WrongNumberOfMatches(usize),
    /// Cannot set opponent without a player id
    #[error("Need a player id for opponent")]
    OpponentIsNotAPlayer,
}

/// Seeds of players
type Seeds = [usize; 2];

/// A match result is a score. For example, I win 2-0
pub type MatchReportedResult = [(i8, i8); 2];

/// Reported result
#[derive(Debug, Clone, Copy)]
pub struct ReportedResult(pub (i8, i8));

impl std::cmp::PartialEq<ReportedResult> for ReportedResult {
    fn eq(&self, other: &ReportedResult) -> bool {
        self.0 .0 == other.0 .0 && self.0 .1 == other.0 .1
    }
}

impl ReportedResult {
    /// Reverse score
    #[must_use]
    pub fn reverse(self) -> Self {
        Self((self.0 .1, self.0 .0))
    }
}

impl std::fmt::Display for ReportedResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.0 .0, self.0 .1)
    }
}

/// Error while parsing match result
#[derive(Error, Debug)]
pub enum MatchResultParsingError {
    /// Could not parse
    #[error("Match could not be parsed")]
    CouldNotParseResult,
}

impl FromStr for ReportedResult {
    type Err = MatchResultParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO use string split with '-' and parse both side
        Ok(match s {
            "2-0" => Self((2, 0)),
            "2-1" => Self((2, 1)),
            "1-2" => Self((1, 2)),
            "0-2" => Self((0, 2)),
            "0-0" => Self((0, 0)),
            _ => return Err(MatchResultParsingError::CouldNotParseResult),
        })
    }
}

/// Players in match
pub type MatchPlayers = [Opponent; 2];

/// A match between two players, resulting in a winner and a loser
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Match {
    /// Identifier of match
    id: Id,
    /// Participants
    players: MatchPlayers,
    /// seeds\[0\]: top seed
    /// seeds\[1\]: bottom seed
    seeds: Seeds,
    /// The winner of this match
    winner: Opponent,
    /// The looser of this match by disqualification
    automatic_looser: Opponent,
    /// Result reported by players
    reported_results: MatchReportedResult,
}

impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\t{} vs {}", self.players[0], self.players[1])?;
        writeln!(f, "winner: {}", self.winner)
    }
}

impl Match {
    /// Clear result from match and returns updated match
    #[must_use]
    pub(crate) fn clear_reported_result(self, player_id: PlayerId) -> Self {
        let mut m = self.clone();
        if let Opponent::Player(p) = &self.players[0] {
            if p.get_id() == player_id {
                m.reported_results[0] = (0, 0);
            }
        }
        if let Opponent::Player(p) = &self.players[1] {
            if p.get_id() == player_id {
                m.reported_results[1] = (0, 0);
            }
        }

        m
    }

    /// Returns true if one of the player has id `player_id`
    #[must_use]
    pub fn contains(&self, player_id: PlayerId) -> bool {
        if let Opponent::Player(p) = &self.players[0] {
            if p.get_id() == player_id {
                return true;
            }
        }
        if let Opponent::Player(p) = &self.players[1] {
            if p.get_id() == player_id {
                return true;
            }
        }

        false
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

    /// Get id of match
    #[must_use]
    pub fn get_id(&self) -> Id {
        self.id
    }

    /// Get looser of match. Loosers are always players
    #[must_use]
    pub fn get_looser(&self) -> Opponent {
        self.automatic_looser.clone()
    }

    /// Get players for this match
    #[must_use]
    pub fn get_players(&self) -> MatchPlayers {
        self.players.clone()
    }

    /// Get seeds of (predicted) player
    #[must_use]
    pub fn get_seeds(&self) -> Seeds {
        self.seeds
    }

    /// Get winner of match. Winners are players
    #[must_use]
    pub fn get_winner(&self) -> Opponent {
        self.winner.clone()
    }

    /// Returns true if player the automatic looser of this match is given
    /// player
    #[must_use]
    pub(crate) fn is_automatic_looser_by_disqualification(&self, player_id: PlayerId) -> bool {
        if let Opponent::Player(loser) = &self.automatic_looser {
            return loser.get_id() == player_id;
        }
        false
    }

    #[must_use]
    /// Returns true if this match is where loser arrives
    pub(crate) fn is_first_loser_match(&self, expected_seed: usize) -> bool {
        (Opponent::Unknown == self.players[0] && self.seeds[0] == expected_seed)
            || (Opponent::Unknown == self.players[1] && self.seeds[1] == expected_seed)
    }

    /// Returns true if match is over
    #[must_use]
    pub fn is_over(&self) -> bool {
        self.players[0] != Opponent::Unknown
            && self.players[1] != Opponent::Unknown
            && (self.winner != Opponent::Unknown || self.automatic_looser != Opponent::Unknown)
    }

    /// Returns true if both opponents are present but a winner has yet to be
    /// declared
    #[must_use]
    pub fn is_playable(&self) -> bool {
        self.winner == Opponent::Unknown
            && self.automatic_looser == Opponent::Unknown
            && self.players[0] != Opponent::Unknown
            && self.players[1] != Opponent::Unknown
    }

    /// Create looser bracket match where opponents are unknown yet
    #[must_use]
    #[cfg(test)]
    pub fn looser_bracket_match(id: Id, seeds: [usize; 2]) -> Self {
        Match {
            id,
            players: [Opponent::Unknown, Opponent::Unknown],
            seeds,
            winner: Opponent::Unknown,
            automatic_looser: Opponent::Unknown,
            reported_results: [(0, 0), (0, 0)],
        }
    }

    /// Returns true if participant of match is disqualified but winner is not
    /// declared
    #[must_use]
    pub fn needs_update_because_of_disqualified_participant(&self) -> bool {
        self.winner == Opponent::Unknown
            && self.automatic_looser != Opponent::Unknown
            && self.players[0] != Opponent::Unknown
            && self.players[1] != Opponent::Unknown
    }

    /// Create new match with two opponents
    ///
    /// Winner is automatically set if bye opponent is set
    ///
    /// # Errors
    /// Returns an error if bye opponent does not have a known opponent
    pub fn new(players: [Opponent; 2], seeds: [usize; 2]) -> Result<Match, Error> {
        if let Opponent::Player(p1_id) = &players[0] {
            if let Opponent::Player(p2_id) = &players[1] {
                if p1_id == p2_id {
                    return Err(Error::SamePlayer);
                }
            }
        }
        Ok(Self {
            id: Id::new_v4(),
            players,
            winner: Opponent::Unknown,
            automatic_looser: Opponent::Unknown,
            seeds,
            reported_results: [(0_i8, 0_i8), (0_i8, 0)],
        })
    }

    /// Create new looser bracket match where opponents are unknown yet
    #[must_use]
    pub fn new_looser_bracket_match(seeds: [usize; 2]) -> Self {
        Match {
            id: Id::new_v4(),
            players: [Opponent::Unknown, Opponent::Unknown],
            seeds,
            winner: Opponent::Unknown,
            automatic_looser: Opponent::Unknown,
            reported_results: [(0, 0), (0, 0)],
        }
    }

    /// Partitions double elimination bracket matches in winner bracket, looser
    /// bracket, grand finals and grand finals reset
    pub(crate) fn partition_double_elimination_matches(
        matches: &[Match],
        n: usize,
    ) -> Result<(Vec<Match>, Vec<Match>, Match, Match), Error> {
        if matches.len() != 2 * n - 1 {
            return Err(Error::WrongNumberOfMatches(matches.len()));
        }
        let total_winner_bracket_matches = n - 1;
        let (winner_bracket, other) = matches.split_at(total_winner_bracket_matches);
        let (grand_finals_reset, other) = other.split_last().expect("grand finals reset");
        let (grand_finals, loser_bracket) = other.split_last().expect("grand finals");
        Ok((
            winner_bracket.to_vec(),
            loser_bracket.to_vec(),
            grand_finals.clone(),
            grand_finals_reset.clone(),
        ))
    }

    /// Set looser of this match (when disqualified)
    ///
    /// # Errors
    /// thrown when looser is not a participant of the match
    pub fn set_looser(self, player_id: PlayerId) -> Result<Self, Error> {
        if !self.contains(player_id) {
            return Err(Error::UnknownPlayer(player_id, self.players));
        }
        let mut loser = Opponent::Unknown;
        if let Opponent::Player(p) = &self.players[0] {
            if p.get_id() == player_id {
                loser = self.players[0].clone();
            }
        }
        if let Opponent::Player(p) = &self.players[1] {
            if p.get_id() == player_id {
                loser = self.players[1].clone();
            }
        }
        Ok(Self {
            automatic_looser: loser,
            ..self
        })
    }

    /// Set opponent of match
    ///
    /// Motivation for this function is to return an error which can be coerced
    /// whereas `set_player` forces you to have an Id
    ///
    /// # Errors
    /// thrown if opponent is not a player
    pub(crate) fn set_opponent(self, p: Player, is_player_1: bool) -> Self {
        assert!(
            !self.contains(p.get_id()),
            "cannot set opponent when already in the match"
        );
        let player = Opponent::Player(p);
        let players = if is_player_1 {
            [player, self.players[1].clone()]
        } else {
            [self.players[0].clone(), player]
        };
        Self { players, ..self }
    }

    #[must_use]
    /// Set player of match and return updated match
    pub fn set_player(self, player: Player, is_player_1: bool) -> Match {
        let player = Opponent::Player(player);
        let players = if is_player_1 {
            [player, self.players[1].clone()]
        } else {
            [self.players[0].clone(), player]
        };
        Match { players, ..self }
    }

    /// Returns true if the stronger seed won
    #[must_use]
    pub(crate) fn stronger_seed_wins(&self) -> bool {
        assert!(self.seeds[0] != self.seeds[1]);
        if self.seeds[0] < self.seeds[1] {
            if let Opponent::Player(p) = &self.players[0] {
                if let Opponent::Player(winner) = &self.winner {
                    return winner.clone() == p.clone();
                }
            }
        } else if let Opponent::Player(p) = &self.players[1] {
            if let Opponent::Player(winner) = &self.winner {
                return winner.clone() == p.clone();
            }
        }
        false
    }

    /// Set match outcome using reported results. Returns updated match, winner
    /// id and looser id
    ///
    /// # Errors
    /// Returns an error if reported scores don't not agree on the winner
    pub fn update_outcome(self) -> Result<(Match, Player, Player), Error> {
        // if there is a disqualified player, try to set the winner
        if let Opponent::Player(dq_player) = self.automatic_looser.clone() {
            if let Opponent::Player(p) = self.players[0].clone() {
                if p.get_id() == dq_player.get_id() {
                    if let Opponent::Player(other_p) = self.players[1].clone() {
                        return Ok((
                            Self {
                                winner: Opponent::Player(other_p.clone()),
                                ..self
                            },
                            other_p,
                            dq_player,
                        ));
                    }
                }
            }
            if let Opponent::Player(p) = self.players[1].clone() {
                if p.get_id() == dq_player.get_id() {
                    if let Opponent::Player(other_p) = self.players[0].clone() {
                        return Ok((
                            Self {
                                winner: Opponent::Player(other_p.clone()),
                                ..self
                            },
                            other_p,
                            dq_player,
                        ));
                    }
                }
            }

            return Err(Error::MissingOpponent(self.players));
        }

        let [(s11, s12), (s21, s22)] = self.reported_results;
        let result_1 = ReportedResult((s11, s12));
        let result_2 = ReportedResult((s21, s22));
        if result_1.reverse() != result_2 {
            return Err(Error::PlayersReportedDifferentMatchOutcome([
                ReportedResult((self.reported_results[0].0, self.reported_results[0].1)),
                ReportedResult((self.reported_results[1].0, self.reported_results[1].1)),
            ]));
        }
        let (winner, loser) = if s11 > s12 && s21 < s22 {
            (self.players[0].clone(), self.players[1].clone())
        } else if s11 < s12 && s21 > s22 {
            (self.players[1].clone(), self.players[0].clone())
        } else {
            return Err(Error::PlayersReportedDifferentMatchOutcome([
                ReportedResult((self.reported_results[0].0, self.reported_results[0].1)),
                ReportedResult((self.reported_results[1].0, self.reported_results[1].1)),
            ]));
        };

        let winner = match winner {
            Opponent::Player(w) => w,
            Opponent::Unknown => return Err(Error::UnknownPlayerWithReportedResults),
        };
        let loser = match loser {
            Opponent::Player(l) => l,
            Opponent::Unknown => return Err(Error::UnknownPlayerWithReportedResults),
        };

        Ok((
            Match {
                id: self.id,
                players: self.players,
                seeds: self.seeds,
                winner: Opponent::Player(winner.clone()),
                automatic_looser: self.automatic_looser,
                reported_results: self.reported_results,
            },
            winner,
            loser,
        ))
    }

    /// Update match result and return updated match
    ///
    /// # Errors
    /// Thrown when referred player is not in the match
    pub fn update_reported_result(
        self,
        player_id: PlayerId,
        result: ReportedResult,
    ) -> Result<Match, Error> {
        let player1 = match &self.players[0] {
            Opponent::Player(p) => p,
            Opponent::Unknown => return Err(Error::MissingOpponent(self.players)),
        };
        let player2 = match &self.players[1] {
            Opponent::Player(p) => p,
            Opponent::Unknown => return Err(Error::MissingOpponent(self.players)),
        };
        if player1.get_id() == player_id {
            let mut reported_results = self.reported_results;
            reported_results[0] = result.0;
            Ok(Match {
                id: self.id,
                players: self.players,
                seeds: self.seeds,
                winner: self.winner,
                automatic_looser: self.automatic_looser,
                reported_results,
            })
        } else if player2.get_id() == player_id {
            let mut reported_results = self.reported_results;
            reported_results[1] = result.0;
            Ok(Match {
                id: self.id,
                players: self.players,
                seeds: self.seeds,
                winner: self.winner,
                automatic_looser: self.automatic_looser,
                reported_results,
            })
        } else {
            Err(Error::UnknownPlayer(player_id, self.players))
        }
    }
}

/// Match representation as received through the network
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct MatchGET {
    /// Match id
    pub id: Id,
    /// Participants: "id name" x2
    pub players: [String; 2],
    /// seeds\[0\]: top seed
    /// seeds\[1\]: bottom seed
    pub seeds: Seeds,
    /// The winner of this match, "?" if unknown
    pub winner: String,
    /// The looser of this match, "?" if unknown
    pub looser: String,
    /// Results reported by players
    pub reported_results: [String; 2],
}

impl MatchGET {
    #[must_use]
    /// Create raw match data object
    pub fn new(
        id: Id,
        players: &[Opponent; 2],
        seeds: Seeds,
        winner: &Opponent,
        looser: &Opponent,
        rr: MatchReportedResult,
    ) -> Self {
        Self {
            id,
            players: [players[0].to_string(), players[1].to_string()],
            seeds,
            winner: winner.to_string(),
            looser: looser.to_string(),
            reported_results: [
                ReportedResult(rr[0]).to_string(),
                ReportedResult(rr[1]).to_string(),
            ],
        }
    }
}

/// Error while converting response from network
#[derive(Error, Debug)]
pub enum MatchParsingError {
    /// Could not parse bracket id for match
    #[error("Match opponent id could not be parsed: {0}")]
    Opponent(#[from] ParsingOpponentError),
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

impl TryFrom<MatchGET> for Match {
    type Error = MatchParsingError;

    fn try_from(m: MatchGET) -> Result<Match, Self::Error> {
        let players = m
            .players
            .into_iter()
            .map(|m| m.parse::<Opponent>())
            .collect::<Result<Vec<Opponent>, ParsingOpponentError>>()?;
        let players: [Opponent; 2] = players.try_into()?;
        let winner = m.winner.parse::<Opponent>()?;
        if winner != Opponent::Unknown && !players.iter().any(|p| *p == winner) {
            return Err(MatchParsingError::UnknownWinner);
        }
        let looser = m.looser.parse::<Opponent>()?;
        if looser != Opponent::Unknown && !players.iter().any(|p| *p == looser) {
            return Err(MatchParsingError::UnknownLooser);
        }
        Ok(Self {
            id: m.id,
            players,
            seeds: m.seeds,
            winner,
            automatic_looser: looser,
            reported_results: [
                m.reported_results[0].parse::<ReportedResult>()?.0,
                m.reported_results[0].parse::<ReportedResult>()?.0,
            ],
        })
    }
}

// NOTE: here because Vec<T> does not implement std::error::Error when used with try_into
impl From<Vec<Opponent>> for MatchParsingError {
    fn from(opponents: Vec<Opponent>) -> Self {
        MatchParsingError::GatherOpponents(opponents)
    }
}

impl From<Match> for MatchGET {
    fn from(m: Match) -> Self {
        Self {
            id: m.id,
            players: m.players.map(|p| p.to_string()),
            seeds: m.seeds,
            winner: m.winner.to_string(),
            looser: m.automatic_looser.to_string(),
            reported_results: [
                ReportedResult(m.reported_results[0]).to_string(),
                ReportedResult(m.reported_results[1]).to_string(),
            ],
        }
    }
}

/// Print player name for opponent. Returns None if player was not found in list
#[must_use]
pub fn print_player_name(o: &Opponent, players: &[Player]) -> Option<String> {
    match o {
        Opponent::Player(p) => players
            .iter()
            .find(|player| player.get_id() == p.get_id())
            .map(Player::get_name),
        Opponent::Unknown => Some(Opponent::Unknown.to_string()),
    }
}

/// Id of match
pub type Id = uuid::Uuid;

/// Response to query: "who is my next opponent"
#[derive(Serialize, Deserialize)]
pub struct NextMatchGET {
    /// Next opponent
    pub opponent: Opponent,
    /// Id of next match
    pub match_id: Id,
    /// Bracket where next match happens
    pub bracket_id: BracketId,
    /// Name of next opponent
    pub player_name: String,
}

/// Raw response to query: "who is my next opponent"
#[derive(Serialize, Deserialize)]
pub struct NextMatchGETResponse {
    /// Next opponent
    pub opponent: String,
    /// Id of next match
    pub match_id: Id,
    /// Bracket where next match happens
    pub bracket_id: BracketId,
    /// Name of next opponent
    pub player_name: String,
}

/// Error while parsing next match
#[derive(Error, Debug, Clone)]
pub enum NextMatchGETParsingError {
    /// Could not parse opponent
    #[error("{0}")]
    Opponent(#[from] ParsingOpponentError),
}

impl TryFrom<NextMatchGETResponse> for NextMatchGET {
    type Error = NextMatchGETParsingError;

    fn try_from(r: NextMatchGETResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            opponent: r.opponent.parse::<Opponent>()?,
            match_id: r.match_id,
            bracket_id: r.bracket_id,
            player_name: r.player_name,
        })
    }
}

/// request for next match
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct NextMatchGETRequest {
    /// Next opponent
    pub player_internal_id: String,
    /// Identifier of the discussion channel from service (for instance: discord)
    pub channel_internal_id: String,
    /// Name of service. See totsugeki_api for a list of supported service
    pub service_type_id: String,
}

impl std::fmt::Display for NextMatchGET {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Next opponent: {} ({})\nMatch ID: {}\nBracket ID: {}",
            self.player_name, self.opponent, self.match_id, self.bracket_id
        )
    }
}

/// Player reports match result
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct PlayerMatchResultPOST {
    /// Player id using service
    pub internal_player_id: String,
    /// Discussion channel id of service
    pub internal_channel_id: String,
    /// Service used to make call
    pub service: String,
    /// Result as reported by the player
    pub result: String,
}

/// Tournament organiser reports match result where player 1 scored result x-y
/// against player 2
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct TournamentOrganiserMatchResultPOST {
    /// Discussion channel id of service
    pub internal_channel_id: String,
    /// Service used to make call
    pub service: String,
    /// Player 1
    pub player1: String,
    /// Result as reported by the player
    pub result: String,
    /// Player 2
    pub player2: String,
}

/// Validate match
#[derive(Serialize)]
pub struct ValidateMatchPOST {
    /// Discussion channel id of service
    pub channel_internal_id: String,
    /// Service used to make call
    pub service_type_id: String,
    /// Result as reported by the player
    pub match_id: String,
}

/// Reponse to reporting result with affected match id and some message
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct ReportResultPOST {
    /// Id of match where result is reported
    pub affected_match_id: Id,
    /// Additionnal message which may contain a warning
    pub message: String,
    /// List of new matches to play after updating the bracket
    pub matches: Vec<MatchGET>,
}

/// Raw response to next match query: Opponent is not parsed
// NOTE: enum implement FromStr and ToString so you don't have to implement
// ToJson trait
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct NextMatchGETResponseRaw {
    /// Next opponent
    pub opponent: String,
    /// Id of next match
    pub match_id: Id,
    /// Bracket where next match happens
    pub bracket_id: BracketId,
    /// Opponent name
    pub player_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_contains_both_players() {
        let p1 = PlayerId::new_v4();
        let player_1 = Opponent::Player(Player {
            id: p1,
            name: "p1".into(),
        });
        let p2 = PlayerId::new_v4();
        let player_2 = Opponent::Player(Player {
            id: p2,
            name: "p2".into(),
        });
        let unknown = PlayerId::new_v4();
        let m = Match::new([player_1, player_2], [1, 2]).expect("match");
        assert!(m.contains(p1));
        assert!(m.contains(p2));
        assert!(!m.contains(unknown));
    }

    #[test]
    fn cannot_create_match_with_same_player() {
        let p = PlayerId::new_v4();
        let player = Opponent::Player(Player {
            id: p,
            name: "some player".into(),
        });
        if let Err(e) = Match::new([player.clone(), player], [1, 2]) {
            match e {
                Error::SamePlayer => {}
                _ => panic!("Expected error SamePlayer but got {e}"),
            }
        } else {
            panic!("Expected error but got none")
        }
    }

    #[test]
    fn stronger_seeds_wins() {
        let p1 = Player::new("p1".into());
        let p2 = Player::new("p2".into());
        let m = Match::new(
            [Opponent::Player(p1.clone()), Opponent::Player(p2.clone())],
            [1, 2],
        )
        .expect("match");
        assert!(!m.is_over());
        let m = m
            .update_reported_result(p1.get_id(), ReportedResult((2, 0)))
            .expect("match p1 result");
        let m = m
            .update_reported_result(p2.get_id(), ReportedResult((0, 2)))
            .expect("match p2 result");
        let (m, _, _) = m.update_outcome().expect("validation");
        assert!(m.is_over());
        assert!(
            m.stronger_seed_wins(),
            "expected p1 with higher seed to win"
        );

        let m = Match::new(
            [Opponent::Player(p1.clone()), Opponent::Player(p2.clone())],
            [1, 2],
        )
        .expect("match");
        assert!(!m.is_over());
        let m = m
            .update_reported_result(p2.get_id(), ReportedResult((2, 0)))
            .expect("match p2 result");
        let m = m
            .update_reported_result(p1.get_id(), ReportedResult((0, 2)))
            .expect("match p1 result");
        let (m, _, _) = m.update_outcome().expect("validation");
        assert!(m.is_over());
        assert!(
            !m.stronger_seed_wins(),
            "expected p2 with lower seed to win"
        );

        let m = Match::new(
            [Opponent::Player(p1.clone()), Opponent::Player(p2.clone())],
            [2, 1],
        )
        .expect("match");
        assert!(!m.is_over());
        let m = m
            .update_reported_result(p1.get_id(), ReportedResult((2, 0)))
            .expect("match p1 result");
        let m = m
            .update_reported_result(p2.get_id(), ReportedResult((0, 2)))
            .expect("match p2 result");
        let (m, _, _) = m.update_outcome().expect("validation");
        assert!(m.is_over());
        assert!(!m.stronger_seed_wins(), "p1 with lower seed to win");

        let m = Match::new(
            [Opponent::Player(p1.clone()), Opponent::Player(p2.clone())],
            [2, 1],
        )
        .expect("match");
        assert!(!m.is_over());
        let m = m
            .update_reported_result(p2.get_id(), ReportedResult((2, 0)))
            .expect("match p1 result");
        let m = m
            .update_reported_result(p1.get_id(), ReportedResult((0, 2)))
            .expect("match p1 result");
        let (m, _, _) = m.update_outcome().expect("validation");
        assert!(m.is_over());
        assert!(
            m.stronger_seed_wins(),
            "expected p2 with higher seed to win"
        );
    }

    #[test]
    fn playable_matches() {
        let p1 = Player::new("p1".into());
        let p2 = Player::new("p2".into());
        let m = Match::new(
            [Opponent::Player(p1.clone()), Opponent::Player(p2.clone())],
            [1, 2],
        )
        .expect("match");
        assert!(m.is_playable());

        let m = Match::new([Opponent::Unknown, Opponent::Player(p2)], [1, 2]).expect("match");
        assert!(!m.is_playable());
        let m = Match::new([Opponent::Player(p1), Opponent::Unknown], [1, 2]).expect("match");
        assert!(!m.is_playable());
        let m = Match::new([Opponent::Unknown, Opponent::Unknown], [1, 2]).expect("match");
        assert!(!m.is_playable());
    }
}
