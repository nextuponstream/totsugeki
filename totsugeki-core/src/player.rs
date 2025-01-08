//! player

use crate::bracket::seeding::Seeding;
use crate::ID;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

/// Player ID
#[derive(Hash, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Ord, PartialOrd, Copy)]
#[allow(clippy::module_name_repetitions)]
pub struct PlayerID(ID);

impl PlayerID {
    /// Create player ID
    #[must_use]
    pub fn create() -> Self {
        Self(ID::new_v4())
    }

    /// New player ID
    #[must_use]
    pub fn new(id: ID) -> Self {
        Self(id)
    }
}

impl FromStr for PlayerID {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = ID::parse_str(s)?;
        Ok(PlayerID(id))
    }
}

impl Display for PlayerID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player ID: {}", self.0)
    }
}

/// A player is referenced by their ID and their username
#[derive(Hash, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Player {
    /// Player identifier
    id: PlayerID,
    /// Player name
    name: String,
}

impl TryFrom<(&str, &str)> for Player {
    type Error = Error;

    fn try_from((id, name): (&str, &str)) -> Result<Self, Self::Error> {
        Ok(Player {
            id: id.parse::<PlayerID>()?,
            name: name.into(),
        })
    }
}

impl From<(PlayerID, String)> for Player {
    fn from((id, name): (PlayerID, String)) -> Self {
        Player { id, name }
    }
}

impl From<(PlayerID, &str)> for Player {
    fn from((id, name): (PlayerID, &str)) -> Self {
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
            id: PlayerID(ID::new_v4()),
            name,
        }
    }

    /// Get player id
    #[must_use]
    pub fn get_id(&self) -> PlayerID {
        self.id
    }

    /// Get player name
    #[must_use]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

// FIXME you can use anonymous structure instead
/// Participants of bracket
///
/// Participants are ordered by seeding position from strongest to weakest
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Participants(Vec<Player>);

/// Error while updating or querying a group of players
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
    Unknown(PlayerID),
}

impl Participants {
    /// Add player to participants
    ///
    /// # Errors
    /// thrown if player is already present
    // FIXME only 1 error variant at play in this method, then extract error
    // enum
    pub fn add_participant(self, new_player: Player) -> Result<Self, Error> {
        if self.0.iter().any(|p| p.get_id() == new_player.get_id()) {
            Err(Error::AlreadyPresent)
        } else {
            let mut updated_participants = self.0;
            updated_participants.push(new_player);
            Ok(Self(updated_participants))
        }
    }

    /// Form participants with provided (id, name) pairs
    ///
    /// # Errors
    /// thrown error if two same players are added
    pub fn from_raw_id(players_to_add: Vec<(String, String)>) -> Result<Participants, Error> {
        let mut players = Participants::default();
        for p in players_to_add {
            let id = p.0.as_str().parse::<PlayerID>()?;
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
    pub fn get(&self, participant_id: PlayerID) -> Option<Player> {
        self.0
            .iter()
            .find(|p| p.get_id() == participant_id)
            .cloned()
    }

    /// Return participants as a list of players
    #[must_use]
    pub fn get_players_list(&self) -> Vec<Player> {
        self.0.clone()
    }

    /// Returns seeding. Default seeding is first participant registered gets
    /// the highest seed
    ///
    /// # Panics
    /// Participant player list is corrupted
    #[must_use]
    pub fn get_seeding(&self) -> Seeding {
        Seeding::new(self.0.iter().map(Player::get_id).collect::<Vec<_>>())
            .expect("seeding from participants")
    }
    /// Returns seeding, which is the players listed by ID
    #[must_use]
    pub fn get_player_list(&self) -> Vec<PlayerID> {
        self.0.iter().map(Player::get_id).collect::<Vec<_>>()
    }

    /// Returns true if both group of participants have the same players,
    /// disregarding order
    #[must_use]
    pub fn have_same_participants(&self, other_group: &Participants) -> bool {
        let mut players = self
            .0
            .clone()
            .iter()
            .map(Player::get_id)
            .collect::<Vec<PlayerID>>();
        players.sort();
        let mut other_players = other_group
            .0
            .clone()
            .iter()
            .map(Player::get_id)
            .collect::<Vec<PlayerID>>();
        other_players.sort();
        players == other_players
    }

    /// Returns `true` if there is no participants
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Number of participants
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Remove participant
    ///
    /// # Errors
    /// thrown if participant does not belong to this group
    #[must_use]
    pub fn remove(self, participant_id: PlayerID) -> Self {
        Self(
            self.0
                .into_iter()
                .filter(|p| p.get_id() != participant_id)
                .collect::<Vec<_>>(),
        )
    }

    /// Add player to participants but does not check if player is already
    /// present in bracket. Use only for fuzzing tests when you can guarantee
    /// you do not add duplicate players.
    ///
    /// # Safety
    /// Adding two same players may result in undefined behavior
    #[must_use]
    pub fn unchecked_add_participant(self, new_player: Player) -> Self {
        let mut updated_participants = self.0;
        updated_participants.push(new_player);
        Self(updated_participants)
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {}", self.id, self.name)
    }
}

impl Display for Participants {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Players:")?;
        for p in &self.0 {
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

impl TryFrom<Vec<(&ID, &String)>> for Participants {
    type Error = Error;

    fn try_from(players: Vec<(&ID, &String)>) -> Result<Self, Self::Error> {
        let mut result = Participants::default();
        for p in players {
            let p = Player {
                id: PlayerID(*p.0),
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
