//! Tournament player definition

use crate::guests::GuestID;
use crate::tournaments::ID;
use crate::users::registration::UserID;
use serde::Deserialize;
use totsugeki_core::player::{Player, PlayerID};

/// Tournament player refer to either a real user or a guest (but NEVER both)
#[derive(Debug, Clone, Deserialize)]
pub struct TournamentPlayer {
    /// Player ID
    pub id: ID,
    /// User ID
    pub user_id: Option<UserID>,
    /// Guest ID
    pub guest_id: Option<GuestID>,
    /// Name of player
    pub name: String,
}

impl From<TournamentPlayer> for Player {
    fn from(value: TournamentPlayer) -> Self {
        Player::from((PlayerID::new(value.id), value.name))
    }
}

/// Cannot create tournament player
#[derive(Debug)]
pub enum Error {
    /// Either user ID or guest ID
    EitherOneOrTheOther(Option<UserID>, Option<GuestID>),
}

impl TournamentPlayer {
    /// Create new tournament player
    pub fn new(
        user_id: Option<UserID>,
        guest_id: Option<GuestID>,
        name: String,
    ) -> Result<TournamentPlayer, Error> {
        if let Some(user_id) = user_id {
            Ok(TournamentPlayer {
                id: ID::new_v4(),
                user_id: Some(user_id),
                guest_id: None,
                name,
            })
        } else if let Some(guest_id) = guest_id {
            Ok(TournamentPlayer {
                id: ID::new_v4(),
                user_id: None,
                guest_id: Some(guest_id),
                name,
            })
        } else {
            Err(Error::EitherOneOrTheOther(user_id, guest_id))
        }
    }

    /// Get user ID of player
    pub fn get_user(&self) -> Option<UserID> {
        self.user_id
    }

    /// Get guest ID of player
    pub fn get_guest(&self) -> Option<GuestID> {
        self.guest_id
    }

    /// Get ID of player
    pub fn get_id(&self) -> ID {
        self.id
    }
}
