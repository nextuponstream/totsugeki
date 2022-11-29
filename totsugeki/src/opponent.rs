//! Opponent

use crate::player::{Id as PlayerId, Player};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Opponent in a match
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub enum Opponent {
    // FIXME use PlayerId instead
    /// A player
    Player(Player),
    /// Opponent has not been decided yet
    #[default]
    Unknown,
}

impl std::fmt::Display for Opponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opponent::Player(p) => write!(f, "{} {}", p.id, p.name),
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
    /// Could not split id from name
    #[error("Could not split Id from name. Please separate Id from name with a single space: {0}")]
    Split(String),
}

impl std::str::FromStr for Opponent {
    type Err = ParsingOpponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "?" {
            Ok(Opponent::Unknown)
        } else {
            let (id, name) = match s.split_once(' ') {
                Some(r) => r,
                None => return Err(ParsingOpponentError::Split(s.into())),
            };
            let id = PlayerId::parse_str(id)?;
            Ok(Opponent::Player(Player {
                id,
                name: name.into(),
            }))
        }
    }
}
