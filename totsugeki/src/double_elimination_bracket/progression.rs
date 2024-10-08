//! Progression of a double elimination bracket

use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::{Id, Match, ReportedResult};
use crate::ID;

/// Error while reporting for a double elimination bracket
#[derive(Debug)]
pub enum DoubleEliminationReportResultError {}

/// All methods to update matches of an ongoing double elimination bracket
pub trait ProgressionDEB {
    /// Tournament organiser reports result for a given match
    fn tournament_organiser_reports_result(
        self,
        match_id: ID,
        player1: ID,
        result: (i8, i8),
        player2: ID,
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>;

    /// Tournament organiser reports result
    ///
    /// NOTE: both players are needed, so it is less ambiguous when reading code:
    /// * p1 2-0 is more ambiguous to read than
    /// * p1 2-0 p2
    ///
    /// Technically, it's unnecessary.
    ///
    /// This method is dangerous because in a double-elimination bracket, it's
    /// possible that a player plays against the same opponent twice, like grand
    /// finals into grand final reset. If a request to update grand finals is
    /// sent twice by accident, then grand finals AND grand final reset match
    /// may get updated. While this is a niche corner case, you may want to use
    /// the safer method `tournament_organiser_reports_result`
    ///
    /// # Panics
    /// When either `player1` or `player2` is unknown
    fn tournament_organiser_reports_result_dangerous(
        self,
        player1: ID,
        result: (i8, i8),
        player2: ID,
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>;

    /// Tournament organiser reports `result` for match where `player` is
    /// involved.
    ///
    /// This method is dangerous because not idempotent
    fn tournament_organiser_reports_result_for_single_player_dangerous(
        self,
        player: ID,
        result: (i8, i8),
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>;

    /// Report result of player.
    ///
    /// This method is dangerous because it is not idempotent: if you sent the result twice when you
    /// meant to send it once, you can accidentally update two matches.
    fn report_result_dangerous(
        &self,
        player_id: ID,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, ID, Vec<Match>), DoubleEliminationReportResultError>;
}

impl ProgressionDEB for DoubleEliminationBracket {
    fn tournament_organiser_reports_result(
        self,
        match_id: ID,
        player1: ID,
        result: (i8, i8),
        player2: ID,
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>
    {
        todo!()
    }

    fn tournament_organiser_reports_result_dangerous(
        self,
        player1: ID,
        result: (i8, i8),
        player2: ID,
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>
    {
        // clear reported results
        let bracket = self.clone().clear_reported_result(player1);
        let bracket = bracket.clear_reported_result(player2);

        // report score as p1
        let result_player_1 = ReportedResult(Some(result));
        let (matches, first_affected_match, _new_matches) =
            bracket.report_result_dangerous(player1, result_player_1.0.expect("result"))?;

        // // report same score as p2
        let bracket = DoubleEliminationBracket::new(
            matches,
            self.seeding.clone(),
            self.automatic_match_validation_mode,
        );

        let (matches, second_affected_match, new_matches) = bracket
            .report_result_dangerous(player2, result_player_1.reverse().0.expect("result"))?;
        //
        // assert_eq!(first_affected_match, second_affected_match);
        //
        // Ok((matches, first_affected_match, new_matches))
        todo!()
    }

    fn tournament_organiser_reports_result_for_single_player_dangerous(
        self,
        player_left: ID,
        result: (i8, i8),
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>
    {
        todo!()
    }

    fn report_result_dangerous(
        &self,
        player_id: ID,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, ID, Vec<Match>), DoubleEliminationReportResultError> {
        todo!()
    }
}
