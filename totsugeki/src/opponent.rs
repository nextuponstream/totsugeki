//! Opponent

use crate::player::{Id as PlayerId, Participants, Player};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Opponent in a match
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default, Copy)]
pub enum Opponent {
    /// Any player is uniquely referred by its ID
    Player(PlayerId),
    /// Opponent has not been decided yet
    #[default]
    Unknown,
}

impl Opponent {
    /// Get name of player if available
    #[must_use]
    pub fn get_name(&self, players: &[(PlayerId, String)]) -> String {
        match self {
            Opponent::Player(id) if players.iter().any(|p| p.0 == *id) => {
                let Some(p) = players.iter().find(|p| p.0 == *id) else {
                    unreachable!("player is missing");
                };
                p.1.clone()
            }
            _ => Opponent::Unknown.to_string(),
        }
    }
}

impl std::fmt::Display for Opponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opponent::Player(p) => write!(f, "{p}"),
            Opponent::Unknown => write!(f, "?"),
        }
    }
}

/// Error while parsing Opponent
#[derive(Error, Debug, Clone)]
pub enum ParsingOpponentError {
    /// Could not parse opponent player ID
    #[error("{0}")]
    Id(#[from] uuid::Error),
}

impl std::str::FromStr for Opponent {
    type Err = ParsingOpponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "?" => Opponent::Unknown,
            _ => Opponent::Player(PlayerId::parse_str(s)?),
        })
    }
}
