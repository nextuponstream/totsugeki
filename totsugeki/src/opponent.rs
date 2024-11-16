//! Opponent

use crate::player::PlayerID;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
/// Your opponent for the next match may not be there
pub struct Opponent(pub Option<PlayerID>);

impl Opponent {
    /// Get name of player if available
    #[must_use]
    pub fn get_name(&self, players: &[(PlayerID, String)]) -> String {
        if let Some(player) = self.0 {
            let Some((p, name)) = players.iter().find(|p| p.0 == player) else {
                unreachable!("player is missing");
            };
            format!("{p} {name}")
        } else {
            "?".into()
        }
    }
}

impl std::fmt::Display for Opponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(p) => write!(f, "{p}"),
            None => write!(f, "?"),
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
            "?" => Self(None),
            _ => Self(Some(s.parse()?)),
        })
    }
}
