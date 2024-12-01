//! Double elimination bracket

use crate::bracket::late_bracket_configuration::LateBracketConfiguration;
use crate::bracket::seeding::Seeding;
use crate::matches::result::MatchFormat;
use crate::matches::{Match, MatchID};
use crate::seeding::double_elimination_seeded_bracket::get_loser_bracket_matches_top_seed_favored;
use crate::validation::AutomaticMatchValidationMode;
use serde::{Deserialize, Serialize};

pub mod disqualification;
mod getters;
pub mod next_opponent;
// FIXME refactor everything double elimination bracket here
mod partition;

pub mod progression;
pub mod reporting;
mod validate_from;

/// Double elimination bracket
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DoubleEliminationBracket {
    // NOTE: not worth using a container. Though I want to do `matches.contains(match_id)`...
    /// Matches
    matches: Vec<Match>,
    /// Seeding
    seeding: Seeding,
    /// Condition for automatic match validation when reports come in
    automatic_match_validation_mode: AutomaticMatchValidationMode,
}

impl DoubleEliminationBracket {
    /// Generate matches for a new bracket using `seeding` and other configuration
    #[must_use]
    pub fn create(
        seeding: Seeding,
        automatic_match_validation_mode: AutomaticMatchValidationMode,
        base_match_format: MatchFormat,
        late_bracket_configuration: Option<LateBracketConfiguration>,
    ) -> Self {
        let mut matches = vec![];
        if seeding.len() >= 3 {
            // FIXME remove unwrap, this should never panic
            let mut winner_bracket_matches =
                crate::seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored(&seeding, base_match_format, late_bracket_configuration);
            matches.append(&mut winner_bracket_matches);
            let mut looser_bracket_matches =
                get_loser_bracket_matches_top_seed_favored(&seeding, base_match_format, None);

            matches.append(&mut looser_bracket_matches);
            let grand_finals: Match = Match::new_empty([1, 2], base_match_format);
            matches.push(grand_finals);
            let grand_finals_reset: Match = Match::new_empty([1, 2], base_match_format);
            matches.push(grand_finals_reset);
        }

        Self {
            matches,
            seeding,
            automatic_match_validation_mode,
        }
    }

    /// Construct double elimination bracket from given data. Typically used
    /// when retrieving from a database.
    ///
    /// # Panics
    /// When a double-elimination bracket cannot be made from `matches` and
    /// `seeding`.
    #[must_use]
    pub fn new(
        matches: Vec<Match>,
        seeding: Seeding,
        automatic_match_validation_mode: AutomaticMatchValidationMode,
    ) -> Self {
        assert!(
            (seeding.is_empty() && matches.is_empty())
                || (!seeding.is_empty() && !matches.is_empty()),
            "no seeding for matches generated"
        );
        let magic = 2_usize * seeding.len();
        assert!(
            seeding.is_empty() || matches.len() == magic - 1,
            "expected 2*n - 1 matches for n players (n > 0)"
        );
        // TODO more assertions
        Self {
            matches,
            seeding,
            automatic_match_validation_mode,
        }
    }

    /// Get matches
    #[must_use]
    pub fn get_matches(&self) -> Vec<Match> {
        self.matches.clone()
    }

    /// Clear all reported results for given match in bracket
    #[must_use]
    fn clear_reported_result(self, match_id: MatchID) -> Self {
        let mut matches = self.matches.clone();
        let match_to_update = matches
            .iter_mut()
            .find(|m| m.id == match_id)
            .expect("found match");
        match_to_update.clear_reported_result();
        Self { matches, ..self }
    }
}
