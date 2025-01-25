//! Guest user definition

use crate::tournaments::ID;
use serde::{Deserialize, Serialize};

/// ID of guest
#[derive(sqlx::Type, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GuestID(ID);

impl Default for GuestID {
    fn default() -> Self {
        Self(ID::new_v4())
    }
}

impl GuestID {
    /// New guest ID
    pub fn new(id: ID) -> Self {
        GuestID(id)
    }
    /// Get ID of guest
    pub fn get(&self) -> ID {
        self.0
    }
}

/// Guests are not real users
pub struct Guest(pub(crate) GuestID, pub(crate) String);

impl Guest {
    /// Create new guest with given `name`
    pub fn new(name: String) -> Self {
        Self(GuestID::default(), name)
    }

    /// Get ID of `guest`
    pub fn get_id(&self) -> GuestID {
        self.0
    }

    /// Get name of guest
    pub fn get_name(&self) -> String {
        self.1.clone()
    }
}
