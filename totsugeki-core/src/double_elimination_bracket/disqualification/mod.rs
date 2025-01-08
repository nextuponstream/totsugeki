//! Disqualify from double elimination bracket

use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::player::PlayerID;

pub mod disqualify_from_bracket;
mod disqualify_from_match;

impl DoubleEliminationBracket {
    /// Returns `true` if player is disqualified
    #[must_use]
    pub fn is_disqualified(&self, player_id: PlayerID) -> bool {
        crate::bracket::matches::is_disqualified(player_id, &self.matches)
    }
}
