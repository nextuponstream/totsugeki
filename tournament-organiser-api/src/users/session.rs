//! User session

use std::fmt::Formatter;

/// All keys in session
pub(crate) enum Keys {
    /// user ID of logged in user
    UserId,
}

// Note could use strum crate for this but do you really want to add a crate just for this?
impl std::fmt::Display for Keys {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let key = match self {
            Keys::UserId => "user_id",
        };
        write!(f, "{key}")
    }
}
