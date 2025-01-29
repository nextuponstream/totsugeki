//! Guest user definition

use crate::ID;

/// Guests are not real users
pub struct Guest(pub(crate) ID, pub(crate) String);

impl Guest {
    /// Create new guest with given `name`
    #[must_use]
    pub fn new(name: String) -> Self {
        Self(ID::new_v4(), name)
    }

    /// Get ID of `guest`
    #[must_use]
    pub fn get_id(&self) -> &ID {
        &self.0
    }

    /// Get name of guest
    #[must_use]
    pub fn get_name(&self) -> &String {
        &self.1
    }
}
