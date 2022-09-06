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
use tracing::debug;

/// Error while creating a match
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
    UnknownPlayer(PlayerId, MatchPlayers),
    /// Cannot update match result because an opponent is missing
    #[error("Cannot report result in a match where opponent is missing. Current players: {} VS {}", .0[0], .0[1])]
    MissingOpponent(MatchPlayers),
    /// Match got into a really bad state where an unknown player has result
    #[error("Error. Unable to proceed further.")]
    UnknownPlayerWithReportedResults,
}

/// Seeds of players
type Seeds = [usize; 2];

/// A match result is a score. For example, I win 2-0
pub type MatchReportedResult = [(i8, i8); 2];

/// Reported result
#[derive(Debug, Clone, Copy)]
pub struct ReportedResult(pub (i8, i8));

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
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Match {
    /// Identifier of match
    id: Id,
    /// Two players from this match. One of the player can be a BYE opponent
    players: MatchPlayers,
    /// seeds\[0\]: top seed
    /// seeds\[1\]: bottom seed
    seeds: Seeds,
    /// The winner of this match
    winner: Opponent,
    /// The looser of this match
    looser: Opponent,
    /// Result reported by players
    reported_results: MatchReportedResult,
}

impl Match {
    /// Get matches to send
    #[must_use]
    pub fn get_sendable_matches(matches: &Vec<Vec<Match>>) -> Vec<Vec<MatchGET>> {
        let mut result = vec![];
        for round in matches {
            let mut result_round = vec![];
            for m in round {
                result_round.push((*m).into());
            }

            result.push(result_round);
        }

        result
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
        let player = Opponent::Player(player_id);
        match self.players[0] {
            Opponent::Player(_) => {}
            Opponent::Unknown => return Err(Error::MissingOpponent(self.players)),
        }
        match self.players[1] {
            Opponent::Player(_) => {}
            Opponent::Unknown => return Err(Error::MissingOpponent(self.players)),
        }
        if self.players[0] == player {
            let mut reported_results = self.reported_results;
            reported_results[0] = result.0;
            Ok(Match {
                id: self.id,
                players: self.players,
                seeds: self.seeds,
                winner: self.winner,
                looser: self.looser,
                reported_results,
            })
        } else if self.players[1] == player {
            let mut reported_results = self.reported_results;
            reported_results[1] = result.0;
            Ok(Match {
                id: self.id,
                players: self.players,
                seeds: self.seeds,
                winner: self.winner,
                looser: self.looser,
                reported_results,
            })
        } else {
            Err(Error::UnknownPlayer(player_id, self.players))
        }
    }

    #[must_use]
    /// Set player of match and return updated match
    pub fn set_player(self, player_id: PlayerId, is_player_1: bool) -> Match {
        let player = Opponent::Player(player_id);
        let players = if is_player_1 {
            [player, self.players[1]]
        } else {
            [self.players[0], player]
        };
        Match {
            id: self.id,
            players,
            seeds: self.seeds,
            winner: self.winner,
            looser: self.looser,
            reported_results: self.reported_results,
        }
    }
}

impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\t{} vs {}", self.players[0], self.players[1])?;
        writeln!(f, "winner: {}", self.winner)
    }
}

impl Match {
    /// Create new match with two opponents.
    /// Expected inputs are:
    /// * `Some(Some(PLAYER_ID))`, when opponent is know
    /// * `Some(None)`, if bye opponent
    /// * `None` if unknown (for instance, final round match)
    ///
    /// Winner is automatically set if bye opponent is set
    ///
    /// # Errors
    /// Returns an error if bye opponent does not have a known opponent
    pub fn new(players: [Opponent; 2], seeds: [usize; 2]) -> Result<Match, Error> {
        Ok(Self {
            id: Id::new_v4(),
            players,
            winner: Opponent::Unknown,
            looser: Opponent::Unknown,
            seeds,
            reported_results: [(0_i8, 0_i8), (0_i8, 0)],
        })
    }

    /// Get winner of match. Winners are players
    #[must_use]
    pub fn get_winner(&self) -> Opponent {
        self.winner
    }

    /// Get looser of match. Loosers are always players
    #[must_use]
    pub fn get_looser(&self) -> Opponent {
        self.looser
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

    /// Get id of match
    #[must_use]
    pub fn get_id(&self) -> Id {
        self.id
    }

    /// Set match outcome using reported results. Returns updated match, seed
    /// of expected winner and winner id
    ///
    /// # Errors
    /// Returns an error if reported scores don't not agree on the winner
    pub fn update_outcome(self) -> Result<(Match, usize, PlayerId), Error> {
        let [(s11, s12), (s21, s22)] = self.reported_results;
        let seed_of_expected_winner = self.get_seeds()[0];
        let winner = if s11 > s12 && s21 < s22 {
            self.players[0]
        } else if s11 < s12 && s21 > s22 {
            self.players[1]
        } else {
            return Err(Error::PlayersReportedDifferentMatchOutcome([
                ReportedResult((self.reported_results[0].0, self.reported_results[0].1)),
                ReportedResult((self.reported_results[1].0, self.reported_results[1].1)),
            ]));
        };

        let winner_id = match winner {
            Opponent::Player(id) => id,
            Opponent::Unknown => return Err(Error::UnknownPlayerWithReportedResults),
        };

        debug!("winner: {winner}");

        Ok((
            Match {
                id: self.id,
                players: self.players,
                seeds: self.seeds,
                winner,
                looser: self.looser,
                reported_results: self.reported_results,
            },
            seed_of_expected_winner,
            winner_id,
        ))
    }
}

/// Match representation as received through the network
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct MatchGET {
    /// Match id
    pub id: Id,
    /// Two players from this match. One of the player can be a BYE opponent
    pub players: [String; 2],
    /// seeds\[0\]: top seed
    /// seeds\[1\]: bottom seed
    pub seeds: Seeds,
    /// The winner of this match, "?" if unknown, "BYE" for BYE opponent
    pub winner: String,
    /// The looser of this match, "?" if unknown, "BYE" for BYE opponent
    pub looser: String,
    /// Results reported by players
    pub reported_results: [String; 2],
}

impl MatchGET {
    #[must_use]
    /// Create raw match data object
    pub fn new(
        id: Id,
        players: [Opponent; 2],
        seeds: Seeds,
        winner: Opponent,
        looser: Opponent,
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
            looser,
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
            looser: m.looser.to_string(),
            reported_results: [
                ReportedResult(m.reported_results[0]).to_string(),
                ReportedResult(m.reported_results[1]).to_string(),
            ],
        }
    }
}

/// Print player name for opponent. Returns None if player was not found in list
#[must_use]
pub fn print_player_name(o: Opponent, players: &[Player]) -> Option<String> {
    match o {
        Opponent::Player(id) => players
            .iter()
            .find(|p| p.get_id() == id)
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

/// Report match result
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct MatchResultPOST {
    /// Player id using service
    pub player_internal_id: String,
    /// Discussion channel id of service
    pub channel_internal_id: String,
    /// Service used to make call
    pub service_type_id: String,
    /// Result as reported by the player
    pub result: String,
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
