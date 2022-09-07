//! player

use crate::bracket::Id as BracketId;
#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// A player is referenced by their ID and their username
#[derive(Hash, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct Player {
    /// Player identifier
    pub id: Id,
    /// Player name
    pub name: String,
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

/// Participants of bracket
///
/// Participants are ordered by seeding position from strongest to weakest
#[derive(Default, Debug, Clone)]
pub struct Participants {
    /// players from this group
    participants: Vec<Player>,
}

/// Error while forming group of players
#[derive(Error, Debug, Eq, PartialEq)]
pub enum Error {
    /// Player already exist in this group of player
    #[error("Player already present in group")]
    AlreadyPresent,
    /// Player id could not be parsed
    #[error("Player id parsing failed")]
    PlayerId(#[from] uuid::Error),
}

impl Participants {
    /// Add player to participants
    ///
    /// # Errors
    /// thrown if player is already present
    pub fn add(&mut self, new_player: Player) -> Result<(), Error> {
        if self
            .participants
            .iter()
            .any(|p| p.get_id() == new_player.get_id())
        {
            Err(Error::AlreadyPresent)
        } else {
            self.participants.push(new_player);
            Ok(())
        }
    }

    /// Number of participants
    #[must_use]
    pub fn len(&self) -> usize {
        self.participants.len()
    }

    /// Returns `true` if there is no participants
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.participants.is_empty()
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
            if let Err(e) = players.add(p) {
                return Err(e);
            }
        }
        Ok(players)
    }

    /// Return participants as a list of players
    #[must_use]
    pub fn get_players_list(self) -> Vec<Player> {
        self.participants
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
            result.add(p)?;
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
            result.add(p)?;
        }
        Ok(result)
    }
}

/// Body of request to get players from active bracket in discussion channel
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
#[cfg_attr(feature = "poem-openapi", oai(rename = "PlayersGET"))]
pub struct GET {
    /// Internal discussion channel ID
    pub internal_discussion_channel_id: String,
    /// String representation of service used (example: Discord)
    pub service: String,
}

/// Response body of players request (see [`GET`])
#[derive(Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct PlayersRaw {
    /// Id of bracket
    pub bracket_id: BracketId,
    /// Players in bracket
    pub players: Vec<Id>,
    /// Players_names in bracket
    pub player_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_two_same_players_returns_error() {
        let same_player = Player::new("same_player".to_string());
        let mut players = Participants::default();
        assert!(players.add(same_player.clone()).is_ok());
        let e = players.add(same_player);
        assert!(e.is_err(), "adding the same player did not return an error");
        match e.as_ref().expect_err("error") {
            Error::AlreadyPresent => {}
            Error::PlayerId(_) => panic!("expected AlreadyPresent but got {e:?}"),
        }
    }
}
