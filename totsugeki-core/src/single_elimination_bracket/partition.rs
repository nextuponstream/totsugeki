//! partition matches of single elimination bracket

use crate::bracket::winner_bracket::winner_bracket;
use crate::bracket::PartitionError;
use crate::matches::Match;
use crate::single_elimination_bracket::SingleEliminationBracket;

impl SingleEliminationBracket {
    /// Partitions matches (round 1 = R1, R2, R3...)
    ///
    /// # Errors
    /// There are not enough players in the bracket
    pub fn partition_by_round(&self) -> Result<Vec<Vec<Match>>, PartitionError> {
        Ok(winner_bracket(self.matches.clone(), &self.seeding))
    }
}
