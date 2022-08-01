//! Two players play a match, resulting in a winner and a loser

use crate::player::Id as PlayerId;
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
        write!(f, "{} vs {}", self.players[0], self.players[1])
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
            players,
            winner,
            looser: Opponent::Unknown,
            seeds,
        })
    }

    /// Get winner of match. Winners are players
    #[must_use]
    pub fn get_winner(&self) -> Option<Opponent> {
        match self.winner {
            Opponent::Player(id) => Some(Opponent::Player(id)),
            Opponent::Bye | Opponent::Unknown => None,
        }
    }

    /// Get looser of match. Loosers are always players
    #[must_use]
    pub fn get_looser(&self) -> Option<Opponent> {
        match self.looser {
            Opponent::Player(id) => Some(Opponent::Player(id)),
            Opponent::Bye | Opponent::Unknown => None,
        }
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
}
