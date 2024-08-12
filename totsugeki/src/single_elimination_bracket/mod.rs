//! Single elimination bracket

mod progression;

use crate::bracket::seeding::Seeding;
use crate::matches::Match;
use crate::seeding::Error as SeedingError;
use crate::single_elimination_bracket::progression::{Progression, Step};
use crate::ID;
use thiserror::Error;

/// Single elimination bracket
pub struct SingleEliminationBracket {
    /// Matches
    matches: Vec<Match>,
    /// Seeding
    seeding: Seeding,
    /// True when a match should not require tournament organiser to be finalized
    automatic_match_progression: bool,
}

/// All errors you might come across when players reports match result
#[derive(Error, Debug)]
pub enum SingleEliminationReportResultError {
    #[error("Cannot join single elimination bracket because of unrecoverable seeding error {0}")]
    /// Seeding is wrong
    UnrecoverableSeedingError(#[from] SeedingError),
    /// Player is unknown, user provided a wrong player
    #[error("Player {0} is unknown")]
    UnknownPlayer(ID),
    /// Tournament is already over
    #[error("Tournament is over")]
    TournamentIsOver,
}

impl SingleEliminationBracket {
    /// Report result for a match in this bracket. Returns updated bracket,
    /// match id where result is reported and new generated matches if
    /// automatic match validation is on.
    ///
    /// # Errors
    /// thrown when result cannot be parsed
    pub fn report_result(
        self,
        player_id: ID,
        result: (i8, i8),
    ) -> Result<(SingleEliminationBracket, ID, Vec<Match>), SingleEliminationReportResultError>
    {
        // let (matches, affected_match_id, new_matches) = report_result(player_id, result)?;
        // let bracket = Self { matches, ..self };
        // bracket.check_all_assertions();
        // Ok((bracket, affected_match_id, new_matches))
        todo!()
    }
}
