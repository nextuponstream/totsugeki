//! Assertion to make against a bracket

use crate::bracket::Bracket;

impl Bracket {
    /// Check all available assertions for bracket
    pub(crate) fn check_all_assertions(&self) {
        self.format
            .get_progression(
                self.matches.clone(),
                &self.participants,
                self.automatic_match_progression,
            )
            .check_all_assertions();
    }
}
