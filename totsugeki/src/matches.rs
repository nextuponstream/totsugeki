//! Two players play a match, resulting in a winner and a loser

use crate::player::Id as PlayerId;
use serde::{Deserialize, Serialize};

/// Error while creating a match
#[derive(Debug, Clone)]
pub enum Error {
    /// Bye opponent cannot be unknown
    MissingOpponentForByeOpponent,
}

/// Opponent can either be a player or the fake opponent of a bye match
// Some(PlayerId) is a player. None is a bye opponent
type Opponent = Option<PlayerId>;

/// A match between two players, resulting in a winner and a loser
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Match {
    /// Two players from this match. One of the player can be a BYE opponent
    players: [Option<Opponent>; 2],
    /// seeds\[0\]: top seed
    /// seeds\[1\]: bottom seed
    seeds: [usize; 2],
    /// The winner of this match
    winner: Option<Opponent>,
    /// The looser of this match
    looser: Option<Opponent>,
}

impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.players[0] {
            Some(p) => match p {
                Some(id) => write!(f, "({}) {id}", self.seeds[0])?,
                None => write!(f, "({}) BYE", self.seeds[0])?,
            },
            None => write!(f, "({}) ?", self.seeds[0])?,
        }
        write!(f, " vs ")?;
        match self.players[1] {
            Some(p) => match p {
                Some(id) => write!(f, "({}) {id}", self.seeds[1])?,
                None => write!(f, "({}) BYE", self.seeds[1])?,
            },
            None => write!(f, "({}) ?", self.seeds[1])?,
        }
        Ok(())
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
    pub fn new(players: [Option<Opponent>; 2], seeds: [usize; 2]) -> Result<Match, Error> {
        let winner = if let Some(None) = players[1] {
            let winner_id = match players[0] {
                Some(id) => id,
                None => return Err(Error::MissingOpponentForByeOpponent),
            };
            Some(winner_id)
        } else {
            None
        };
        Ok(Self {
            players,
            winner,
            looser: None,
            seeds,
        })
    }

    /// Get winner of match
    #[must_use]
    pub fn get_winner(&self) -> Option<PlayerId> {
        match self.winner {
            Some(winner) => winner,
            None => None,
        }
    }

    /// Get looser of match
    #[must_use]
    pub fn get_looser(&self) -> Option<PlayerId> {
        match self.looser {
            Some(looser) => looser,
            None => None,
        }
    }
}
