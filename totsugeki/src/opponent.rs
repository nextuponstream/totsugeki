//! Opponent

use crate::player::Id as PlayerId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Opponent in a match
#[derive(Debug, Copy, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub enum Opponent {
    /// A player
    Player(PlayerId),
    /// Opponent has not been decided yet
    #[default]
    Unknown,
}

impl std::fmt::Display for Opponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opponent::Player(id) => write!(f, "{id}"),
            Opponent::Unknown => write!(f, "?"),
        }
    }
}

/// Error while parsing Opponent
#[derive(Error, Debug, Clone)]
pub enum ParsingOpponentError {
    /// Could not parse opponent id
    #[error("{0}")]
    Id(#[from] uuid::Error),
}

impl std::str::FromStr for Opponent {
    type Err = ParsingOpponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "?" => Opponent::Unknown,
            _ => Opponent::Player(PlayerId::try_from(s)?),
        })
    }
}
