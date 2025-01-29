//! Single elimination bracket

mod disqualify_from_bracket;
mod next_opponent_in_bracket;
mod partition;
pub mod progression;

use crate::bracket::late_bracket_configuration::LateBracketConfiguration;
use crate::bracket::seeding::Seeding;
use crate::matches::result::{MatchFormat, Score};
use crate::matches::Match;
use crate::opponent::Opponent;
use crate::seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored;
use crate::validation::AutomaticMatchValidationMode;
use crate::ID;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Single elimination bracket
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SingleEliminationBracket {
    /// Matches
    matches: Vec<Match>,
    /// Seeding
    seeding: Seeding,
    /// Validation mode. May trigger validation after a reported result
    automatic_match_validation_mode: AutomaticMatchValidationMode,
}

/// Input is invalid because of current bracket state
#[derive(Error, Debug)]
pub enum SingleEliminationReportResultError {
    /// Tournament is already over
    ///
    /// Player ID is valid, but there is no matches to play anyway
    #[error("Tournament is over")]
    TournamentIsOver,
    /// Player is disqualified
    ///
    /// Player ID is valid but disqualified player are not allowed to report
    #[error("Player {0} is disqualified")]
    ForbiddenDisqualified(ID),
    /// No match to play for player
    ///
    /// May happen if tournament organiser validated right before player did for the same match
    #[error("There is no matches for player {0}")]
    NoMatchToPlay(ID),
}

impl SingleEliminationBracket {
    /// Get matches
    #[must_use]
    pub fn get_matches(&self) -> Vec<Match> {
        self.matches.clone()
    }

    /// Generate matches for a new bracket using `seeding` and other configuration
    #[must_use]
    pub fn create(
        seeding: Seeding,
        automatic_match_progression: AutomaticMatchValidationMode,
        match_format: MatchFormat,
        late_bracket_configuration: Option<LateBracketConfiguration>,
    ) -> Self {
        let matches = get_balanced_round_matches_top_seed_favored(
            &seeding,
            match_format,
            late_bracket_configuration,
        );

        Self {
            matches,
            seeding,
            automatic_match_validation_mode: automatic_match_progression,
        }
    }

    /// New single elimination bracket
    ///
    /// # Panics
    /// When a well-formed single-elimination bracket cannot be made from
    /// `matches` and `seeding`
    #[must_use]
    pub fn new(
        seeding: Seeding,
        matches: Vec<Match>,
        automatic_match_progression: AutomaticMatchValidationMode,
    ) -> Self {
        for player in seeding.get() {
            assert!(
                matches
                    .iter()
                    .any(|m| m.players.contains(&Opponent(Some(player)))),
                "player {player} was not found in matches. Is matches data corrupt?"
            );
        }
        // TODO more assertions

        Self {
            matches,
            seeding,
            automatic_match_validation_mode: automatic_match_progression,
        }
    }

    /// Seeding of bracket
    #[must_use]
    pub fn get_seeding(&self) -> Seeding {
        self.seeding.clone()
    }

    /// Report result for a match in this bracket
    ///
    /// If automatic match validation is off, then only returns bracket
    ///
    /// If automatic validation is on, when no player has reported yet a result
    /// for the match so far, returns the bracket. Otherwise, returns updated
    /// bracket, match id where result is reported and new generated matches
    ///
    /// Returns updated bracket, match ID where score was applied and newly
    /// updated matches
    ///
    /// # Errors
    /// thrown when result cannot be parsed or a disqualified player reports
    /// # Panics
    /// When `player_id` is unknown
    pub fn report_result(
        self,
        player_id: &ID,
        result: Score,
    ) -> Result<(SingleEliminationBracket, ID, Vec<Match>), SingleEliminationReportResultError>
    {
        assert!(
            self.seeding.contains(player_id),
            "Unknown player {player_id}"
        );
        if self.is_over() {
            return Err(SingleEliminationReportResultError::TournamentIsOver);
        }
        if self.is_disqualified(player_id) {
            return Err(SingleEliminationReportResultError::ForbiddenDisqualified(
                *player_id,
            ));
        }
        let old_matches = self.matches_to_play();
        let match_to_update = self
            .get_matches()
            .into_iter()
            .find(|m| m.contains(player_id) && *m.get_winner() == Opponent(None));
        let seeding = self.seeding.clone();
        let automatic_match_progression = self.automatic_match_validation_mode;
        match match_to_update {
            Some(m) => {
                let affected_match_id = m.get_id();
                let matches =
                    self.update_player_reported_match_result(affected_match_id, result, player_id)?;
                let bracket =
                    SingleEliminationBracket::new(seeding, matches, automatic_match_progression);

                let bracket = match automatic_match_progression {
                    AutomaticMatchValidationMode::Strict => bracket,
                    AutomaticMatchValidationMode::Flexible => {
                        let updated_match = bracket
                            .get_matches()
                            .into_iter()
                            .find(|mm| mm.id == m.id)
                            .expect("updated match");
                        if updated_match.has_all_player_reports() {
                            bracket.validate_match_result(affected_match_id).0
                        } else {
                            bracket
                        }
                    }
                    AutomaticMatchValidationMode::Lax => {
                        bracket.validate_match_result(affected_match_id).0
                    }
                };

                let new_matches = bracket
                    .matches_to_play()
                    .iter()
                    .filter(|m| !old_matches.iter().any(|old_m| old_m.get_id() == m.get_id()))
                    .map(Clone::clone)
                    .collect();
                Ok((bracket, *affected_match_id, new_matches))
            }
            None => Err(SingleEliminationReportResultError::NoMatchToPlay(
                *player_id,
            )),
        }
    }

    /// Clear previous reported result for `player_id`
    fn clear_reported_result(self, player_id: &ID) -> Self {
        debug_assert!(
            self.matches
                .iter()
                .filter(|m| m.contains(player_id) && *m.get_winner() == Opponent(None))
                .count()
                <= 1
        );
        let match_to_update = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && *m.get_winner() == Opponent(None));
        match match_to_update {
            Some(m_to_clear) => {
                let m_to_clear = (*m_to_clear).clear_reported_result_from(player_id);

                let matches = self
                    .matches
                    .into_iter()
                    .map(|m| {
                        if m.get_id() == m_to_clear.get_id() {
                            m_to_clear
                        } else {
                            m
                        }
                    })
                    .collect();
                Self { matches, ..self }
            }
            None => self,
        }
    }

    /// Get validation
    #[must_use]
    pub fn get_automatic_validation(&self) -> AutomaticMatchValidationMode {
        self.automatic_match_validation_mode
    }
}
