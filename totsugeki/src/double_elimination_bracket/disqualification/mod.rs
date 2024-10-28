//! Disqualify from double elimination bracket

use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::ID;

pub mod disqualify_from_bracket;
mod disqualify_from_match;

impl DoubleEliminationBracket {
    /// Returns `true` if player is disqualified
    pub fn is_disqualified(&self, player_id: ID) -> bool {
        crate::bracket::matches::is_disqualified(player_id, &self.matches)
    }
}
