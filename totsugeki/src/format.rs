//! Format of bracket

use serde::{Deserialize, Serialize};

/// All bracket formats
#[derive(PartialEq, Eq, Copy, Clone, Deserialize, Serialize, Debug)]
pub enum Format {
    /// Single elimination tournament
    SingleElimination,
    // TODO add other style of tournament
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::SingleElimination => write!(f, "single-elimination"),
        }
    }
}

impl std::str::FromStr for Format {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single-elimination" => Ok(Format::SingleElimination),
            _ => Err(ParsingError::Unknown(s.to_string())),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::SingleElimination // TODO set to DoubleElimination when implemented
    }
}

/// Parsing error for Format type
#[derive(Debug)]
pub enum ParsingError {
    /// Unknown format was provided
    Unknown(String),
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::Unknown(format) => writeln!(
                f,
                "Unknown bracket format: \"{format}\". Please try another format such as: \"{}\"",
                Format::default()
            ),
        }
    }
}
