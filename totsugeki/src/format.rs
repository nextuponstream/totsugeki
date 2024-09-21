//! Format of bracket

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    bracket::matches::{
        double_elimination_format::Step as DE_Step, single_elimination_format::Step as SE_Step,
        Progression,
    },
    matches::Match,
    player::{Id as PlayerId, Participants, Player},
    seeding::{
        double_elimination_seeded_bracket::get_loser_bracket_matches_top_seed_favored,
        single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
        Error as SeedingError,
    },
};

/// All bracket formats
#[derive(PartialEq, Eq, Copy, Clone, Deserialize, Serialize, Debug)]
pub enum Format {
    /// Players are eliminated after their first loss
    SingleElimination,
    /// Players are eliminated after their second loss
    DoubleElimination,
}

impl Format {
    /// Generate matches according to the current format
    ///
    /// # Errors
    /// thrown when math overflow happens
    pub fn generate_matches(self, seeding: &[PlayerId]) -> Result<Vec<Match>, SeedingError> {
        Ok(match self {
            Format::SingleElimination => get_balanced_round_matches_top_seed_favored(seeding)?,
            Format::DoubleElimination => {
                let mut matches = vec![];
                let mut winner_bracket_matches =
                    get_balanced_round_matches_top_seed_favored(seeding)?;
                matches.append(&mut winner_bracket_matches);
                let mut looser_bracket_matches =
                    get_loser_bracket_matches_top_seed_favored(seeding)?;
                matches.append(&mut looser_bracket_matches);
                let grand_finals: Match = Match::new_empty([1, 2]);
                matches.push(grand_finals);
                let grand_finals_reset: Match = Match::new_empty([1, 2]);
                matches.push(grand_finals_reset);
                matches
            }
        })
    }

    // FIXME remove abstraction. Putting stuff on the heap may not be necessary
    /// Returns progression implementation for this bracket format
    ///
    /// # Panics
    /// if match generation of given format cannot generate match
    #[must_use]
    pub fn get_progression(
        &self,
        matches: Vec<Match>,
        seeding: &Participants,
        automatic_progression: bool,
    ) -> Box<dyn Progression> {
        match self {
            Format::SingleElimination => Box::new(SE_Step::new(
                matches,
                &seeding.get_seeding(),
                automatic_progression,
            )),
            Format::DoubleElimination => Box::new(
                DE_Step::new(
                    Some(matches),
                    seeding
                        .get_players_list()
                        .iter()
                        .map(Player::get_id)
                        .collect(),
                    automatic_progression,
                )
                .expect("double elimination bracket state"),
            ),
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::SingleElimination => write!(f, "single-elimination"),
            Format::DoubleElimination => write!(f, "double-elimination"),
        }
    }
}

impl std::str::FromStr for Format {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single-elimination" => Ok(Format::SingleElimination),
            "double-elimination" => Ok(Format::DoubleElimination),
            _ => Err(ParsingError::Unknown(s.to_string())),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::DoubleElimination
    }
}

/// Parsing error for Format type
#[derive(Error, Debug)]
pub enum ParsingError {
    /// Unknown format was provided
    #[error(
        "Unknown bracket format: \"{0}\". Please try another format such as: \"{}\"",
        Format::default()
    )]
    Unknown(String),
}
