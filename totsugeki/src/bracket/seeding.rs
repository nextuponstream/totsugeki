//! Update seeding of bracket

use crate::{
    bracket::{Bracket, Error as BracketError},
    player::{Id as PlayerId, Participants, Player},
    seeding::seed,
    ID,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Seeding is an ordered list of player. All players IDs are guaranteed unique
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Seeding(Vec<PlayerId>);

impl Default for Seeding {
    fn default() -> Self {
        Seeding(vec![])
    }
}

/// Error while creating seeding
#[derive(Error, Debug, PartialEq)]
pub enum SeedingError {
    /// Duplicate player
    #[error("Duplicate player {0}")]
    DuplicatePlayer(PlayerId),
}

impl Seeding {
    /// Creates a unique player list, ordered for seeding
    pub fn new(player_ids: Vec<ID>) -> Result<Self, SeedingError> {
        let mut set = HashSet::new();
        for player_id in &player_ids {
            if !set.insert(player_id) {
                return Err(SeedingError::DuplicatePlayer(*player_id));
            }
        }
        Ok(Self(player_ids))
    }

    /// Get seeding
    pub fn get(&self) -> Vec<ID> {
        self.0.clone()
    }

    /// Contains player
    pub fn contains(&self, player_id: ID) -> bool {
        self.0.contains(&player_id)
    }

    /// Number of players
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Bracket {
    /// Update seeding with players ordered by seeding position and generate
    /// matches
    ///
    /// # Errors
    /// thrown when provided players do not match current players in bracket
    pub fn update_seeding(self, players: &[PlayerId]) -> Result<Self, BracketError> {
        if self.accept_match_results {
            return Err(BracketError::Started(self.id, String::new()));
        }

        let mut player_group = Participants::default();
        for sorted_player in players {
            let players = self.get_participants().get_players_list();
            let Some(player) = players.iter().find(|p| p.get_id() == *sorted_player) else {
                return Err(BracketError::UnknownPlayer(
                    *sorted_player,
                    self.participants.clone(),
                    self.id,
                ));
            };
            player_group = player_group.add_participant(player.clone())?;
        }
        let participants = seed(&self.seeding_method, player_group, self.participants)?;
        let matches = self.format.generate_matches(
            &participants
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect::<Vec<_>>(),
        )?;
        Ok(Self {
            participants,
            matches,
            ..self
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bracket::builder::Builder,
        format::Format,
        matches::{Id as MatchId, Match},
        opponent::Opponent,
        player::Error as PlayerError,
        seeding::Error as OldSeedingError,
    };

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
            Err(SeedingError::DuplicatePlayer(duplicate_id))
        )
    }
}
