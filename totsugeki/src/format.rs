//! Format of bracket

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::bracket::seeding::Seeding;
use crate::{
    matches::Match,
    seeding::{
        double_elimination_seeded_bracket::get_loser_bracket_matches_top_seed_favored,
        single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
        Error as SeedingError,
    },
};

// TODO rename TournamentFormat
// TODO add enum TournamentFormatWithData so it's easier for library users to handle all formats
//  with a match statement
// TODO check if TournamentFormat and TournamentFormatWithData should coexist
/// All tournament formats
#[derive(PartialEq, Eq, Copy, Clone, Deserialize, Serialize, Debug)]
pub enum Format {
    /// Players are eliminated after their first loss
    SingleEliminationBracket,
    /// Players are eliminated after their second loss
    DoubleEliminationBracket,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::SingleEliminationBracket => write!(f, "single-elimination"),
            Format::DoubleEliminationBracket => write!(f, "double-elimination"),
        }
    }
}

impl std::str::FromStr for Format {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single-elimination" => Ok(Format::SingleEliminationBracket),
            "double-elimination" => Ok(Format::DoubleEliminationBracket),
            _ => Err(ParsingError::Unknown(s.to_string())),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::DoubleEliminationBracket
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
