//! player

use crate::player::Id as PlayerId;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// A player is referenced by their ID and their username
#[derive(Hash, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Player {
    /// Player identifier
    id: Id,
    /// Player name
    name: String,
}

impl TryFrom<(&str, &str)> for Player {
    type Error = Error;

    fn try_from((id, name): (&str, &str)) -> Result<Self, Self::Error> {
        Ok(Player {
            id: id.parse::<Id>()?,
            name: name.into(),
        })
    }
}

impl From<(Id, String)> for Player {
    fn from((id, name): (Id, String)) -> Self {
        Player { id, name }
    }
}

impl From<(Id, &str)> for Player {
    fn from((id, name): (Id, &str)) -> Self {
        Player {
            id,
            name: name.to_string(),
        }
    }
}

impl Player {
    /// Create new player
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            id: Id::new_v4(),
            name,
        }
    }

    /// Get player id
    #[must_use]
    pub fn get_id(&self) -> Id {
        self.id
    }

    /// Get player name
    #[must_use]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

/// Player identifier
pub type Id = Uuid;

// FIXME you can use anonymous structure instead
/// Participants of bracket
///
/// Participants are ordered by seeding position from strongest to weakest
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Participants {
    /// players from this group
    participants: Vec<Player>,
}

/// Error while forming or querying group of players
#[derive(Error, Debug, Eq, PartialEq)]
pub enum Error {
    /// Player already exist in this group of player
    #[error("Player already present in group")]
    AlreadyPresent,
    /// Player id could not be parsed
    #[error("Player id parsing failed")]
    PlayerId(#[from] uuid::Error),
    /// Referenced player is unknown in this group of participants
    #[error("Player {0} is not in this group")]
    Unknown(PlayerId),
}

impl Participants {
    /// Add player to participants
    ///
    /// # Errors
    /// thrown if player is already present
    pub fn add_participant(self, new_player: Player) -> Result<Self, Error> {
        if self
            .participants
            .iter()
            .any(|p| p.get_id() == new_player.get_id())
        {
            Err(Error::AlreadyPresent)
        } else {
            let mut updated_participants = self.participants;
            updated_participants.push(new_player);
            Ok(Self {
                participants: updated_participants,
            })
        }
    }

    /// Form participants with provided (id, name) pairs
    ///
    /// # Errors
    /// thrown error if two same players are added
    pub fn from_raw_id(players_to_add: Vec<(String, String)>) -> Result<Participants, Error> {
        let mut players = Participants::default();
        for p in players_to_add {
            let id = Id::parse_str(&p.0)?;
            let p = Player { id, name: p.1 };
            players = match players.add_participant(p) {
                Ok(updated_players) => updated_players,
                Err(e) => return Err(e),
            };
        }
        Ok(players)
    }

    /// Returns player if present
    #[must_use]
    pub fn get(&self, participant_id: PlayerId) -> Option<Player> {
        self.participants
            .iter()
            .find(|p| p.get_id() == participant_id)
            .cloned()
    }

    /// Return participants as a list of players
    #[must_use]
    pub fn get_players_list(&self) -> Vec<Player> {
        self.participants.clone()
    }

    /// Returns seeding, which is the players listed by ID
    #[must_use]
    pub fn get_seeding(&self) -> Vec<PlayerId> {
        self.participants
            .iter()
            .map(Player::get_id)
            .collect::<Vec<_>>()
    }

    /// Returns true if both group of participants have the same players,
    /// disregarding order
    #[must_use]
    pub fn have_same_participants(&self, other_group: &Participants) -> bool {
        let mut players = self
            .participants
            .clone()
            .iter()
            .map(Player::get_id)
            .collect::<Vec<Id>>();
        players.sort();
        let mut other_players = other_group
            .participants
            .clone()
            .iter()
            .map(Player::get_id)
            .collect::<Vec<Id>>();
        other_players.sort();
        players == other_players
    }

    /// Returns `true` if there is no participants
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.participants.is_empty()
    }

    /// Number of participants
    #[must_use]
    pub fn len(&self) -> usize {
        self.participants.len()
    }

    /// Remove participant
    ///
    /// # Errors
    /// thrown if participant does not belong to this group
    #[must_use]
    pub fn remove(self, participant_id: PlayerId) -> Self {
        Self {
            participants: self
                .participants
                .into_iter()
                .filter(|p| p.get_id() != participant_id)
                .collect::<Vec<_>>(),
        }
    }

    /// Add player to participants but does not check if player is already
    /// present in bracket. Use only for fuzzing tests when you can guarantee
    /// you do not add duplicate players.
    ///
    /// # Safety
    /// Adding two same players may result in undefined behavior
    #[must_use]
    pub fn unchecked_add_participant(self, new_player: Player) -> Self {
        let mut updated_participants = self.participants;
        updated_participants.push(new_player);
        Self {
            participants: updated_participants,
        }
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {}", self.id, self.name)
    }
}

impl std::fmt::Display for Participants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Players:")?;
        for p in &self.participants {
            writeln!(f, "{p}")?;
        }
        Ok(())
    }
}

impl TryFrom<Vec<Player>> for Participants {
    type Error = Error;

    fn try_from(players: Vec<Player>) -> Result<Self, Self::Error> {
        let mut result = Participants::default();
        for p in players {
            result = result.add_participant(p)?;
        }
        Ok(result)
    }
}

impl TryFrom<Vec<(&Id, &String)>> for Participants {
    type Error = Error;

    fn try_from(players: Vec<(&Id, &String)>) -> Result<Self, Self::Error> {
        let mut result = Participants::default();
        for p in players {
            let p = Player {
                id: *p.0,
                name: p.1.to_string(),
            };
            result = result.add_participant(p)?;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_two_same_players_returns_error() {
        let same_player = Player::new("same_player".to_string());
        let players = Participants::default();
        let players = players
            .add_participant(same_player.clone())
            .expect("players");
        match players.add_participant(same_player) {
            Err(Error::AlreadyPresent) => {}
            Err(e) => panic!("expected AlreadyPresent but got {e:?}"),
            Ok(_) => panic!("expected error but got none"),
        }
    }
}
