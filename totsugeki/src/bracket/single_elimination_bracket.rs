//! Single elimination bracket

use crate::bracket::Bracket;
use crate::format::Format;
use crate::matches::Match;

use super::winner_bracket::winner_bracket;
use super::PartitionError;

/// Single elimination bracket variant
#[derive(Debug)]
pub struct Variant {
    /// Some bracket
    bracket: Bracket,
}

/// Error with double elimination brackets
#[derive(Debug)]
pub enum TryIntoError {
    /// Expected format to be double-elimination
    ExpectedSingleEliminationFormat,
}

impl TryFrom<Bracket> for Variant {
    type Error = TryIntoError;

    fn try_from(bracket: Bracket) -> Result<Self, Self::Error> {
        if bracket.format != Format::SingleElimination {
            return Err(TryIntoError::ExpectedSingleEliminationFormat);
        }

        Ok(Variant { bracket })
    }
}

impl Variant {
    /// Returns bracket partitionned by round
    ///
    /// # Errors
    /// Returns an error when there is less than 3 players in the bracket
    pub fn partition_by_round(&self) -> Result<Vec<Vec<Match>>, PartitionError> {
        let wb = winner_bracket(self.bracket.matches.clone(), &self.bracket.participants);

        Ok(wb)
    }
}
