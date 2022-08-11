//! Two players play a match, resulting in a winner and a loser

use crate::{
    bracket::Id as BracketId,
    player::{Id as PlayerId, Player},
};
#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Error while creating a match
#[derive(Debug, Clone)]
pub enum Error {
    /// Bye opponent cannot be unknown
    MissingOpponentForByeOpponent,
    /// Not found
    NotFound,
    /// Players reported different match outcome
    PlayersReportedDifferentMatchOutcome([ReportedResult; 2]),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingOpponentForByeOpponent => write!(f, "Bye player has no opponent"),
            Error::NotFound => write!(f, "Match not found"),
            Error::PlayersReportedDifferentMatchOutcome(r) => {
                write!(
                    f,
                    "Players reported different match outcomes: {} and {} were reported",
                    r[0], r[1]
                )
            }
        }
    }
}

/// Opponent in a match
#[derive(Debug, Copy, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub enum Opponent {
    /// A player
    Player(PlayerId),
    /// Bye opponent (automatic win)
    Bye,
    /// Opponent has not been decided yet
    #[default]
    Unknown,
}

impl std::fmt::Display for Opponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opponent::Player(id) => write!(f, "{id}"),
            Opponent::Bye => write!(f, "BYE"),
            Opponent::Unknown => write!(f, "?"),
        }
    }
}

/// Error while parsing Opponent
#[derive(Debug, Clone)]
pub enum ParsingOpponentError {
    /// Id
    Id(uuid::Error),
}

impl std::fmt::Display for ParsingOpponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingOpponentError::Id(e) => e.fmt(f),
        }
    }
}

impl std::str::FromStr for Opponent {
    type Err = ParsingOpponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "?" => Opponent::Unknown,
            "BYE" => Opponent::Bye,
            _ => Opponent::Player(PlayerId::try_from(s)?),
        })
    }
}

impl From<uuid::Error> for ParsingOpponentError {
    fn from(e: uuid::Error) -> Self {
        Self::Id(e)
    }
}

/// The two players for this match
type MatchPlayers = [Opponent; 2];

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
#[derive(Debug)]
pub enum MatchResultParsingError {
    /// Could not parse
    CouldNotParseResult,
}

impl std::fmt::Display for MatchResultParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchResultParsingError::CouldNotParseResult => {
                writeln!(f, "Match could not be parsed")
            }
        }
    }
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

/// A match between two players, resulting in a winner and a loser
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Match {
    /// Identifier of match
    id: Id,
    /// Two players from this match. One of the player can be a BYE opponent
    pub players: MatchPlayers,
    /// seeds\[0\]: top seed
    /// seeds\[1\]: bottom seed
    seeds: Seeds,
    /// The winner of this match
    pub winner: Opponent,
    /// The looser of this match
    pub looser: Opponent,
    /// Result reported by players
    pub reported_results: MatchReportedResult,
}

impl Match {
    /// Get matches to send
    #[must_use]
    pub fn get_sendable_matches(matches: &Vec<Vec<Match>>) -> Vec<Vec<MatchGET>> {
        let mut result = vec![];
        for round in matches {
            let mut result_round = vec![];
            for m in round {
                result_round.push(m.clone().into());
            }

            result.push(result_round);
        }

        result
    }
}

impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // writeln!(f, "{}", self.id)?;
        writeln!(f, "\t{} vs {}", self.players[0], self.players[1])
        // writeln!(f, "\t{} vs {}", self.seeds[0], self.seeds[1])
        // writeln!(f, "{}", self.winner)
        // writeln!(f, "{}", self.looser)
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
        // let winner = if let Some(None) = players[1] {
        //     let winner_id = match players[0] {
        //         Some(id) => id,
        //         None => return Err(Error::MissingOpponentForByeOpponent),
        //     };
        //     Some(winner_id)
        // } else {
        //     None
        // };
        let winner = if let Opponent::Bye = players[1] {
            match players[0] {
                Opponent::Player(id) => Opponent::Player(id),
                Opponent::Bye | Opponent::Unknown => {
                    return Err(Error::MissingOpponentForByeOpponent)
                }
            }
        } else {
            Opponent::Unknown
        };
        Ok(Self {
            id: Id::new_v4(),
            players,
            winner,
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

    /// Create match from parameters
    ///
    /// # Errors
    /// This function returns an error when the match is invalid
    /// # Panics
    /// not implemented...
    pub fn from(
        id: Id,
        players: [Opponent; 2],
        seeds: [usize; 2],
        winner: Opponent,
        looser: Opponent,
        reported_results: MatchReportedResult,
    ) -> Result<Match, Error> {
        Ok(Self {
            id,
            players,
            seeds,
            winner,
            looser,
            reported_results,
        })
    }

    /// Get id of match
    #[must_use]
    pub fn get_id(&self) -> Id {
        self.id
    }

    /// Set match outcome using reported results. Returns seed of expected winner
    ///
    /// # Errors
    /// Returns an error if reported scores don't not agree on the winner
    pub fn set_outcome(&mut self) -> Result<usize, Error> {
        let [(s11, s12), (s21, s22)] = self.reported_results;

        if s11 > s12 && s21 < s22 {
            self.winner = self.players[0];
            Ok(self.get_seeds()[0])
        } else if s11 < s12 && s21 > s22 {
            self.winner = self.players[1];
            Ok(self.get_seeds()[0])
        } else {
            Err(Error::PlayersReportedDifferentMatchOutcome([
                ReportedResult((self.reported_results[0].0, self.reported_results[0].1)),
                ReportedResult((self.reported_results[1].0, self.reported_results[1].1)),
            ]))
        }
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

/// Error while converting response from network
#[derive(Debug)]
pub enum MatchParsingError {
    /// Could not parse bracket id for match
    Opponent(OpponentParsingError),
    /// Could not gather opponents for a match
    GatherOpponents(Vec<Opponent>),
    /// Reported result could not be parsed
    ReportedResult(MatchResultParsingError),
}

impl std::fmt::Display for MatchParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchParsingError::Opponent(e) => {
                write!(f, "Match opponent id could not be parsed: {e}")
            }
            MatchParsingError::GatherOpponents(o) => {
                write!(f, "Could not use opponents to generate match. Is there two opponents?\nOpponents: {o:?}")
            }
            MatchParsingError::ReportedResult(e) => {
                writeln!(f, "Reported results could not be parsed: {e}")
            }
        }
    }
}

impl TryFrom<MatchGET> for Match {
    type Error = MatchParsingError;

    fn try_from(m: MatchGET) -> Result<Match, Self::Error> {
        let players = m
            .players
            .into_iter()
            .map(Opponent::try_from)
            .collect::<Result<Vec<Opponent>, OpponentParsingError>>()?;
        let players: [Opponent; 2] = players.try_into()?;
        Ok(Self {
            id: m.id,
            players,
            seeds: m.seeds,
            winner: Opponent::try_from(m.winner)?,
            looser: Opponent::try_from(m.looser)?,
            reported_results: [
                m.reported_results[0].parse::<ReportedResult>()?.0,
                m.reported_results[0].parse::<ReportedResult>()?.0,
            ],
        })
    }
}

impl From<MatchResultParsingError> for MatchParsingError {
    fn from(e: MatchResultParsingError) -> Self {
        Self::ReportedResult(e)
    }
}

/// Error while parsing opponent
#[derive(Debug)]
pub enum OpponentParsingError {
    /// Opponent player ID is invalid
    InvalidId,
}

impl std::fmt::Display for OpponentParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpponentParsingError::InvalidId => write!(f, "Opponent ID is invalid"),
        }
    }
}

impl TryFrom<String> for Opponent {
    type Error = OpponentParsingError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(match s.as_str() {
            "BYE" => Opponent::Bye,
            "?" => Opponent::Unknown,
            _ => match PlayerId::parse_str(&s) {
                Ok(id) => Opponent::Player(id),
                Err(_e) => return Err(Self::Error::InvalidId),
            },
        })
    }
}

impl From<OpponentParsingError> for MatchParsingError {
    fn from(e: OpponentParsingError) -> Self {
        Self::Opponent(e)
    }
}

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
        Opponent::Bye => Some(Opponent::Bye.to_string()),
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
#[derive(Debug, Clone)]
pub enum NextMatchGETParsingError {
    /// Could not parse opponent
    Opponent(ParsingOpponentError),
}

impl std::error::Error for NextMatchGETParsingError {}

impl std::fmt::Display for NextMatchGETParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NextMatchGETParsingError::Opponent(e) => e.fmt(f),
        }
    }
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

impl From<ParsingOpponentError> for NextMatchGETParsingError {
    fn from(e: ParsingOpponentError) -> Self {
        Self::Opponent(e)
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
