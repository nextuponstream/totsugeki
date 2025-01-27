//! Tournament player definition

use crate::ID;
use serde::Deserialize;
use totsugeki_core::player::Player;

/// Tournament player refer to either a real user or a guest (but NEVER both)
#[derive(Debug, Clone, Deserialize)]
pub struct TournamentPlayer {
    /// Player ID
    pub id: ID,
    /// User ID
    pub user_id: Option<ID>,
    /// Guest ID
    pub guest_id: Option<ID>,
    /// Name of player
    pub name: String,
    /// Seeding of player
    pub seeding: i16,
}

impl From<TournamentPlayer> for Player {
    fn from(value: TournamentPlayer) -> Self {
        Player::from((value.id, value.name))
    }
}

impl TournamentPlayer {
    /// Instantiate tournament player from `user_id`
    pub fn new_user(user_id: ID, name: String, seeding: i16) -> TournamentPlayer {
        TournamentPlayer {
            id: ID::new_v4(),
            user_id: Some(user_id),
            guest_id: None,
            name,
            seeding,
        }
    }

    /// Instantiate tournament player for new guest
    pub fn new_guest(name: String, seeding: i16) -> TournamentPlayer {
        TournamentPlayer {
            id: ID::new_v4(),
            user_id: None,
            guest_id: Some(ID::new_v4()),
            name,
            seeding,
        }
    }

    /// Get user ID of player
    pub fn get_user(&self) -> Option<ID> {
        self.user_id
    }

    /// Get guest ID of player
    pub fn get_guest(&self) -> Option<ID> {
        self.guest_id
    }

    /// Get ID of player
    pub fn get_id(&self) -> ID {
        self.id
    }
}
