//! Reporting result for a double elimination bracket

use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::{Match, MatchID, MatchResult};
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
    pub fn tournament_organiser_reports_result(
        self,
        match_id: MatchID,
        player1: PlayerID,
        bracket_result: MatchResult,
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

        let matches_where_player1_is_playing: Vec<Match> = self
            .matches
            .clone()
            .into_iter()
            .filter(|m| m.contains(player1) && !m.is_over())
            .collect();
        assert!(
            matches_where_player1_is_playing.len() <= 1,
            "player 1 {player1} is involved in only 1 match but they are involved in {matches_where_player1_is_playing:?}",
        );
        let matches_where_player2_is_playing: Vec<Match> = self
            .clone()
            .matches
            .into_iter()
            .filter(|m| m.contains(player2) && !m.is_over())
            .collect();
        assert!(
            matches_where_player2_is_playing.len() <= 1,
            "player 2 {player2} is involved in only 1 match but they are involved in {:?}",
            matches_where_player2_is_playing
        );

        let m = self
            .matches
            .iter()
            .find(|m| m.id == match_id)
            .expect("match");

        todo!()
    }
}
