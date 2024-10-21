//! Get various information about double elimination bracket

use crate::bracket::seeding::Seeding;
use crate::double_elimination_bracket::DoubleEliminationBracket;

impl DoubleEliminationBracket {
    /// Get seeding of bracket
    pub fn get_seeding(&self) -> Seeding {
        self.seeding.clone()
    }
}
