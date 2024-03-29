//! All getters and setters method of bracket

use crate::{
    format::Format, matches::Match, player::Participants, seeding::Method as SeedingMethod,
};

use super::{Bracket, Id};

impl Bracket {
    /// Bar new participants from entering bracket
    #[must_use]
    pub fn close(self) -> Self {
        Self {
            is_closed: true,
            ..self
        }
    }

    /// Return bracket format
    #[must_use]
    pub fn get_format(&self) -> Format {
        self.format
    }

    /// Get id of bracket
    #[must_use]
    pub fn get_id(&self) -> Id {
        self.bracket_id
    }

    /// Returns matches
    #[must_use]
    pub fn get_matches(&self) -> Vec<Match> {
        self.matches.clone()
    }

    /// Get name of bracket
    #[must_use]
    pub fn get_name(&self) -> String {
        self.bracket_name.clone()
    }

    /// Get participants of bracket
    #[must_use]
    pub fn get_participants(&self) -> Participants {
        self.participants.clone()
    }

    /// Returns seeding method
    #[must_use]
    pub fn get_seeding_method(&self) -> SeedingMethod {
        self.seeding_method
    }

    /// Returns true if match are validated automatically whenever possible
    #[must_use]
    pub fn is_validating_matches_automatically(&self) -> bool {
        self.automatic_match_progression
    }
}
