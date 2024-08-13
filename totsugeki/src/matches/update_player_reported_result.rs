//! Logic when player reports a result

use crate::matches::MatchPlayers;
use crate::ID;
use thiserror::Error;

/// Updating match using player report is impossible
#[derive(Error, Debug)]
pub enum Error {
    /// Match cannot be played
    #[error("Missing opponent, match {0} cannot be played")]
    MissingOpponent(ID, MatchPlayers),
    /// Player does not belong in match
    #[error("Player {1} does not belong in match {0}")]
    UnknownPlayer(ID, ID, MatchPlayers),
}

impl From<Error> for super::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::MissingOpponent(_, players) => Self::MissingOpponent(players),
            Error::UnknownPlayer(_, player, players) => Self::UnknownPlayer(player, players),
        }
    }
}
