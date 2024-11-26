//! Update seeding of bracket

use crate::player::PlayerID;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Seeding is an ordered list of player. All players IDs are guaranteed unique
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Seeding(Vec<PlayerID>);

/// Provided input is unsuitable for seeding
#[derive(Error, Debug, PartialEq)]
pub enum SeedingError {
    /// Duplicate player
    #[error("Duplicate player {0}")]
    DuplicatePlayer(PlayerID),
}

impl Seeding {
    /// Creates a unique player list, ordered for seeding
    ///
    /// # Errors
    /// Player list is unsuitable for seeding
    pub fn new(player_ids: Vec<PlayerID>) -> Result<Self, SeedingError> {
        let mut set = HashSet::new();
        for player_id in &player_ids {
            if !set.insert(player_id) {
                return Err(SeedingError::DuplicatePlayer(player_id.clone()));
            }
        }
        Ok(Self(player_ids))
    }

    /// Get seeding
    pub fn get(&self) -> Vec<PlayerID> {
        self.0.clone()
    }

    /// Contains player
    #[must_use]
    pub fn contains(&self, player_id: PlayerID) -> bool {
        self.0.contains(&player_id)
    }

    /// Number of players
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if no player is seeded
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_many_players() {
        let players = vec![PlayerID::create(), PlayerID::create()];
        assert!(Seeding::new(players).is_ok())
    }
    #[test]
    fn seeding_throws_error_for_duplicate_id() {
        let duplicate_id = PlayerID::create();
        let players = vec![
            PlayerID::create(),
            PlayerID::create(),
            duplicate_id,
            duplicate_id,
        ];
        assert_eq!(
            Seeding::new(players),
            Err(SeedingError::DuplicatePlayer(duplicate_id))
        )
    }
}
