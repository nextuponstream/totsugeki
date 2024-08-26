//! Single elimination bracket

mod progression;

use crate::bracket::seeding::Seeding;
use crate::matches::Match;
use crate::seeding::Error as SeedingError;
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
    /// Match is unknown, user provided a wrong match
    #[error("Match {0} is unknown")]
    UnknownMatch(ID),
    /// Tournament is already over
    #[error("Tournament is over")]
    TournamentIsOver,
    /// Player is disqualified
    #[error("Player {0} is disqualified")]
    ForbiddenDisqualified(ID),
    /// No match to play for player
    #[error("There is no matches for player {0}")]
    NoMatchToPlay(ID),
    /// Missing opponent
    #[error("Missing opponent")]
    MissingOpponent(),
}

/// Cannot generate single elimination bracket
#[derive(Error, Debug)]
pub enum SingleEliminationBracketGenerationError {
    /// Unknown
    #[error("Seeding does not contain player {0} present in match {1}")]
    UnknownPlayer(ID, ID),
}

impl SingleEliminationBracket {
    /// New single elimination bracket
    pub fn new(
        seeding: Seeding,
        matches: Vec<Match>,
        automatic_match_progression: bool,
    ) -> Result<Self, SingleEliminationBracketGenerationError> {
        // NOTE: I really don't like taking `matches` without verifying anything whatsoever
        // FIXME add some assertions, could save from a grave mistake, example: any players not in
        // seeding found in matches should cause an unrecoverable error
        // NOTE: could make it "fumble proof" by only recording reports, but then you have to
        // recompute the bracket at every turn. Then it's not efficient. Just use the database for
        // what it is, saving intermediate state
        todo!()
        // Ok(Self {
        //     seeding,
        //     matches,
        //     automatic_match_progression,
        // })
    }

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
