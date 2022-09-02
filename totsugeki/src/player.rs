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

/// Ordered list of players. Used to seed a bracket
#[derive(Default, Debug, Clone)]
pub struct Players {
    /// players from this group
    players: Vec<Player>,
}

/// Error while interacting with players
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Player already exist in this group of player
    AlreadyPresent,
    /// Player id could not be parsed
    PlayerId(uuid::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyPresent => writeln!(f, "Player already present in group"),
            Error::PlayerId(_e) => writeln!(f, "Player id parsing failed"),
        }
    }
}

impl From<uuid::Error> for Error {
    fn from(e: uuid::Error) -> Self {
        Self::PlayerId(e)
    }
}

impl Players {
    /// Add player to bracket
    ///
    /// # Errors
    /// Returns an error if player already exists in this group
    pub fn add(&mut self, new_player: Player) -> Result<(), Error> {
        if self
            .players
            .iter()
            .any(|p| p.get_id() == new_player.get_id())
        {
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

    /// Form player group from raw player ids
    ///
    /// # Errors
    /// Returns an error if two same players are added
    pub fn from_raw_id(players_to_add: Vec<(String, String)>) -> Result<Players, Error> {
        let mut players = Players::default();
        for p in players_to_add {
            let id = Id::parse_str(&p.0)?;
            let p = Player { id, name: p.1 };
            if let Err(e) = players.add(p) {
                return Err(e);
            }
        }
        Ok(players)
    }

    /// Return players ids as a list
    #[must_use]
    pub fn get_players_list(self) -> Vec<Player> {
        self.players
    }

    /// Returns true if both group of players contains the same players
    /// disregarding order
    #[must_use]
    pub fn contains_same_players(&self, other_group: &Players) -> bool {
        let mut players = self
            .players
            .clone()
            .iter()
            .map(Player::get_id)
            .collect::<Vec<Id>>();
        players.sort();
        let mut other_players = other_group
            .players
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

impl std::fmt::Display for Players {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Players:")?;
        for p in &self.players {
            writeln!(f, "{p}")?;
        }
        Ok(())
    }
}

impl TryFrom<Vec<Player>> for Players {
    type Error = Error;

    fn try_from(players: Vec<Player>) -> Result<Self, Self::Error> {
        let mut result = Players::default();
        for p in players {
            result.add(p)?;
        }
        Ok(result)
    }
}

impl TryFrom<Vec<(&Id, &String)>> for Players {
    type Error = Error;

    fn try_from(players: Vec<(&Id, &String)>) -> Result<Self, Self::Error> {
        let mut result = Players::default();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_two_same_players_returns_error() {
        let same_player = Player::new("same_player".to_string());
        let mut players = Players::default();
        assert!(players.add(same_player.clone()).is_ok());
        let e = players.add(same_player);
        assert!(e.is_err(), "adding the same player did not return an error");
        match e.as_ref().expect_err("error") {
            Error::AlreadyPresent => {}
            _ => panic!("expected AlreadyPresent but got {e:?}"),
        }
    }
}
