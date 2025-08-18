//! Update seeding of bracket

use crate::ID;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Seeding is an ordered list of player. All players IDs are guaranteed unique
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Seeding(Vec<ID>);

/// Provided input is unsuitable for seeding
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Duplicate player
    #[error("Duplicate player {0}")]
    DuplicatePlayer(ID),
}

impl Seeding {
    /// Creates a unique player list, ordered for seeding
    ///
    /// # Errors
    /// Player list is unsuitable for seeding
    pub fn new(player_ids: Vec<ID>) -> Result<Self, Error> {
        let mut set = HashSet::new();
        for player_id in &player_ids {
            if !set.insert(player_id) {
                return Err(Error::DuplicatePlayer(*player_id));
            }
        }
        Ok(Self(player_ids))
    }

    /// Get seeding
    #[must_use]
    pub fn get(&self) -> Vec<ID> {
        self.0.clone()
    }

    /// Contains player
    #[must_use]
    pub fn contains(&self, player_id: &ID) -> bool {
        self.0.contains(player_id)
    }

    /// Number of players
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if no player is seeded
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_many_players() {
        let players = vec![ID::new_v4(), ID::new_v4()];
        assert!(Seeding::new(players).is_ok())
    }
    #[test]
    fn seeding_throws_error_for_duplicate_id() {
        let duplicate_id = ID::new_v4();
        let players = vec![ID::new_v4(), ID::new_v4(), duplicate_id, duplicate_id];
        assert_eq!(
            Seeding::new(players),
            Err(Error::DuplicatePlayer(duplicate_id))
        )
    }
}
