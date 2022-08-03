//! Two players play a match, resulting in a winner and a loser

use crate::{
    bracket::Id as BracketId,
    player::{Id as PlayerId, Player},
};
use serde::{Deserialize, Serialize};

/// Error while creating a match
#[derive(Debug, Clone)]
pub enum Error {
    /// Bye opponent cannot be unknown
    MissingOpponentForByeOpponent,
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

/// The two players for this match
type MatchPlayers = [Opponent; 2];

/// Seeds of players
type Seeds = [usize; 2];

/// A match between two players, resulting in a winner and a loser
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
    ) -> Result<Match, Error> {
        Ok(Self {
            id,
            players,
            seeds,
            winner,
            looser,
        })
    }

    /// Get id of match
    #[must_use]
    pub fn get_id(&self) -> Id {
        self.id
    }
}

/// Match representation as received through the network
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct MatchGET {
    /// Match id
    id: Id,
    /// Two players from this match. One of the player can be a BYE opponent
    players: [String; 2],
    /// seeds\[0\]: top seed
    /// seeds\[1\]: bottom seed
    seeds: Seeds,
    /// The winner of this match, "?" if unknown, "BYE" for BYE opponent
    winner: String,
    /// The looser of this match, "?" if unknown, "BYE" for BYE opponent
    looser: String,
}

/// Error while converting response from network
#[derive(Debug)]
pub enum MatchParsingError {
    /// Could not parse bracket id for match
    Opponent(OpponentParsingError),
    /// Could not gather opponents for a match
    GatherOpponents(Vec<Opponent>),
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
        })
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
}

impl std::fmt::Display for NextMatchGET {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Next opponent: {}\nMatch ID: {}\nBracket ID: {}",
            self.opponent, self.match_id, self.bracket_id
        )
    }
}
