//! player

#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
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

/// Players
#[derive(Default, Debug, Clone)]
pub struct Players {
    /// players from this group
    players: Vec<Id>,
}

/// Error while interacting with players
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Player already exist in this group of player
    AlreadyPresent,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyPresent => writeln!(f, "Player already present in group"),
        }
    }
}

impl Players {
    /// Add player to bracket
    ///
    /// # Errors
    /// Returns an error if player already exists in this group
    pub fn add(&mut self, new_player: Id) -> Result<(), Error> {
        if self.players.iter().any(|p| p == &new_player) {
            Err(Error::AlreadyPresent)
        } else {
            self.players.push(new_player);
            Ok(())
        }
    }

    /// Number of players
    #[must_use]
    pub fn len(&self) -> usize {
        self.players.len()
    }

    /// Returns `true` if there is no players
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// Form player group from `players`
    ///
    /// # Errors
    /// Returns an error if two same players are added
    pub fn from(players_to_add: Vec<Id>) -> Result<Players, Error> {
        let mut players = Players::default();
        for p in players_to_add {
            if let Err(e) = players.add(p) {
                return Err(e);
            }
        }
        Ok(players)
    }

    /// Return players
    #[must_use]
    pub fn get_players(self) -> Vec<Id> {
        self.players
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_two_same_players_returns_error() {
        let same_player = Id::new_v4();
        let mut players = Players::default();
        assert!(players.add(same_player).is_ok());
        let e = players.add(same_player);
        assert!(e.is_err(), "adding the same player did not return an error");
        match e.as_ref().expect_err("error") {
            Error::AlreadyPresent => {} // _ => panic!("wrong error, expected AlreadPresent, got: {e:?}"),
        }
    }
}
