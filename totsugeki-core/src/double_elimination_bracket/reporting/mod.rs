//! Reporting result for a double elimination bracket

use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::result::Score;
use crate::matches::{Match, MatchID, ReportedResult};
use crate::player::PlayerID;
use thiserror::Error;

/// Report for match
#[derive(Debug, Error)]
pub enum MatchReportError {
    /// Already reported
    #[error("match has already been reported")]
    AlreadyReported,
}

impl DoubleEliminationBracket {
    /// Tournament organiser reports result for a given match. Returns updated
    /// bracket and newly playable matches
    ///
    /// # Errors
    /// FIXME add test Reporting twice the same results
    /// # Panics
    /// When data is corrupted
    pub fn tournament_organiser_reports_result(
        self,
        match_id: MatchID,
        player1: PlayerID,
        score: Score,
        player2: PlayerID,
    ) -> Result<(DoubleEliminationBracket, Vec<Match>), MatchReportError> {
        assert!(
            self.seeding.contains(player1),
            "{player1} does not belong in bracket"
        );
        assert!(
            self.seeding.contains(player2),
            "{player2} does not belong in bracket"
        );

        let old_playable_matches = self.matches_to_play();
        let mut matches = self.matches.clone();

        let match_to_update = matches
            .iter_mut()
            .find(|m| m.id == match_id)
            .expect("match to update");
        if match_to_update.is_over() {
            return Err(MatchReportError::AlreadyReported);
        }
        match_to_update.clear_reported_result();

        let bracket = DoubleEliminationBracket::new(
            matches,
            self.seeding.clone(),
            self.automatic_match_validation_mode,
        );

        let result_player_1 = ReportedResult(Some(score));
        let (matches, first_affected_match, _new_matches) = bracket
            .report_result_dangerous(player1, result_player_1.0.expect("result"))
            .expect("matches");

        // report same score as p2
        let bracket = DoubleEliminationBracket::new(
            matches,
            self.seeding.clone(),
            self.automatic_match_validation_mode,
        );

        // TODO just set reported results and validate
        let (matches, second_affected_match, _new_matches) = bracket
            .report_result_dangerous(player2, result_player_1.reverse().0.expect("result"))
            .expect("reported result");

        let bracket = DoubleEliminationBracket::new(
            matches,
            self.seeding.clone(),
            self.automatic_match_validation_mode,
        );

        assert_eq!(first_affected_match, second_affected_match);

        let new_playable_matches = bracket
            .matches_to_play()
            .into_iter()
            .filter(|m| old_playable_matches.iter().any(|old_m| old_m.id == m.id))
            .collect::<Vec<Match>>();

        Ok((bracket, new_playable_matches))
    }
}
