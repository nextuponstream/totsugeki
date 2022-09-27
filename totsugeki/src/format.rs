//! Format of bracket

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    matches::Match,
    opponent::Opponent,
    player::Participants,
    seeding::{
        double_elimination_seeded_bracket::get_looser_bracket_matches_top_seed_favored,
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
    // TODO add other style of tournament
}

impl Format {
    /// Return matches for this bracket format
    ///
    /// # Errors
    /// thrown when math overflow happens
    pub fn get_matches(self, participants: &Participants) -> Result<Vec<Match>, SeedingError> {
        Ok(match self {
            Format::SingleElimination => get_balanced_round_matches_top_seed_favored(participants)?,
            Format::DoubleElimination => {
                let mut matches = vec![];
                let mut winner_bracket_matches =
                    get_balanced_round_matches_top_seed_favored(participants)?;
                matches.append(&mut winner_bracket_matches);
                let mut looser_bracket_matches =
                    get_looser_bracket_matches_top_seed_favored(participants)?;
                matches.append(&mut looser_bracket_matches);
                let grand_finals: Match =
                    Match::new([Opponent::Unknown, Opponent::Unknown], [1, 2])
                        .expect("grand finals");
                matches.push(grand_finals);
                let grand_finals_reset: Match =
                    Match::new([Opponent::Unknown, Opponent::Unknown], [1, 2])
                        .expect("grand finals reset");
                matches.push(grand_finals_reset);
                matches
            }
        })
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
