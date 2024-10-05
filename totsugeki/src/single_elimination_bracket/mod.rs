//! Single elimination bracket

mod disqualify_from_bracket;
mod next_opponent_in_bracket;
pub mod progression;

use crate::bracket::matches::{Error, Progression};
use crate::bracket::seeding::Seeding;
use crate::matches::Match;
use crate::opponent::Opponent;
use crate::opponent::Opponent::Player;
use crate::seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored;
use crate::seeding::Error as SeedingError;
use crate::single_elimination_bracket::progression::ProgressionSEB;
use crate::ID;
use thiserror::Error;

/// Single elimination bracket
#[derive(Clone)]
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
    /// No match to play for player. May happen if tournament organiser validated right before
    /// player did for the same match
    #[error("There is no matches for player {0}")]
    NoMatchToPlay(ID),
    /// Missing opponent
    #[error("Missing opponent")]
    MissingOpponent(),
}

impl SingleEliminationBracket {
    /// Get matches
    pub fn get_matches(&self) -> Vec<Match> {
        self.matches.clone()
    }

    /// Generate matches for bracket using `seeding`  and other configuration
    pub fn create(seeding: Seeding, automatic_match_progression: bool) -> Self {
        let matches = get_balanced_round_matches_top_seed_favored(seeding.clone())
            .expect("initial matches generated");

        Self {
            seeding,
            matches,
            automatic_match_progression,
        }
    }

    /// New single elimination bracket
    ///
    /// # Panics
    /// When player in seeding is not in any of the matches
    pub fn new(seeding: Seeding, matches: Vec<Match>, automatic_match_progression: bool) -> Self {
        for player in seeding.get() {
            assert!(
                matches
                    .iter()
                    .find(|m| m.players.contains(&Player(player)))
                    .is_some(),
                "player {player} was not found in matches. Is matches data corrupt?"
            );
        }

        Self {
            seeding,
            matches,
            automatic_match_progression,
        }
    }

    /// Seeding of bracket
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
    /// # Errors
    /// thrown when result cannot be parsed or a disqualified player reports
    /// # Panics
    /// When `player_id` is unknown
    pub fn report_result(
        self,
        player_id: ID,
        result: (i8, i8),
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
                player_id,
            ));
        }
        let old_matches = self.matches_to_play();
        let match_to_update = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let seeding = self.seeding.clone();
        let automatic_match_progression = self.automatic_match_progression;
        match match_to_update {
            Some(m) => {
                let affected_match_id = m.get_id();
                let matches =
                    self.update_player_reported_match_result(affected_match_id, result, player_id)?;
                let bracket =
                    SingleEliminationBracket::new(seeding, matches, automatic_match_progression);

                let bracket = if automatic_match_progression {
                    bracket.validate_match_result(affected_match_id).0
                } else {
                    bracket
                };

                let new_matches = bracket
                    .matches_to_play()
                    .iter()
                    .filter(|m| !old_matches.iter().any(|old_m| old_m.get_id() == m.get_id()))
                    .map(Clone::clone)
                    .collect();
                Ok((bracket, affected_match_id, new_matches))
            }
            None => Err(SingleEliminationReportResultError::NoMatchToPlay(player_id)),
        }
    }

    /// Clear previous reported result for `player_id`
    fn clear_reported_result(self, player_id: ID) -> Self {
        debug_assert!(
            self.matches
                .iter()
                .filter(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
                .count()
                <= 1
        );
        let match_to_update = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        match match_to_update {
            Some(m_to_clear) => {
                let m_to_clear = (*m_to_clear).clear_reported_result(player_id);

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
}
