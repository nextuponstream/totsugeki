//! Double elimination bracket

use crate::bracket::matches::update_bracket_with;
use crate::bracket::seeding::Seeding;
use crate::matches::Match;
use crate::opponent::Opponent;
use crate::seeding::double_elimination_seeded_bracket::get_loser_bracket_matches_top_seed_favored;
use crate::validation::AutomaticMatchValidationMode;

mod disqualification;
mod getters;
pub mod next_opponent;
// FIXME refactor everything double elimination bracket here

pub mod progression;

/// Double elimination bracket
#[derive(Clone, Debug)]
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
    pub fn create(
        seeding: Seeding,
        automatic_match_validation_mode: AutomaticMatchValidationMode,
    ) -> Self {
        // FIXME remove unwrap, this should never panic
        let mut matches = vec![];
        let mut winner_bracket_matches =
            crate::seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored(seeding.clone()).unwrap();
        matches.append(&mut winner_bracket_matches);
        let mut looser_bracket_matches =
            get_loser_bracket_matches_top_seed_favored(&seeding.get()).unwrap();

        matches.append(&mut looser_bracket_matches);
        let grand_finals: Match = Match::new_empty([1, 2]);
        matches.push(grand_finals);
        let grand_finals_reset: Match = Match::new_empty([1, 2]);
        matches.push(grand_finals_reset);

        Self {
            seeding,
            automatic_match_validation_mode,
            matches,
        }
    }

    /// Construct double elimination bracket from given data. Typically used
    /// when retrieving from a database.
    ///
    /// # Panics
    /// When a double-elimination bracket cannot be made from `matches` and
    /// `seeding`.
    pub fn new(
        matches: Vec<Match>,
        seeding: Seeding,
        automatic_match_validation_mode: AutomaticMatchValidationMode,
    ) -> Self {
        assert!(
            (seeding.len() == 0 && matches.len() == 0) || (seeding.len() > 0 && matches.len() > 0),
            "no seeding for matches generated"
        );
        let magic = 2_usize * seeding.len();
        assert_eq!(
            matches.len(),
            magic - 1,
            "expected 2*n - 1 matches for n players"
        );
        // TODO more assertions
        Self {
            matches,
            seeding,
            automatic_match_validation_mode,
        }
    }

    /// Get matches
    pub fn get_matches(&self) -> Vec<Match> {
        self.matches.clone()
    }

    fn clear_reported_result(self, player_id: crate::player::Id) -> Self {
        let matches_to_update = self
            .matches
            .clone()
            .into_iter()
            .filter(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
            .collect::<Vec<Match>>();
        assert!(
            matches_to_update.len() <= 1,
            "player has to play at most 1 match but found {}",
            matches_to_update.len()
        );
        match matches_to_update.len() {
            1 => {
                let match_to_update = matches_to_update[0];
                let m_to_clear = match_to_update.clear_reported_result(player_id);
                let matches = update_bracket_with(&self.matches, &m_to_clear);

                Self { matches, ..self }
            }
            0 => self,
            _ => unreachable!(),
        }
    }
}
