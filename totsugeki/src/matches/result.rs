//! Match result

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Score of match. Might be intermediate result (like 0-0, or 0-1 in first to 2
/// match format)
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Score(pub u8, pub u8);

/// Match result
#[derive(Debug)]
pub struct MatchResult(Score, MatchFormat);

/// Match formats
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct MatchFormat(u8);

impl Display for MatchFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FT{}", self.0)
    }
}

impl Default for MatchFormat {
    fn default() -> Self {
        Self(2)
    }
}

/// Match format creation error
#[derive(Error, Debug)]
pub enum MatchFormatError {
    /// Invalid match format
    #[error("Invalid match format: First to {0}")]
    Invalid(u8),
}
impl MatchFormat {
    /// First to two
    ///
    /// Commonly seen in large double-elimination bracket because of bracket schedule
    pub fn ft2() -> Self {
        Self(2)
    }

    /// First to three
    ///
    /// Commonly seen in large in double-elimination bracket for
    /// * top8/top6 in large tournament
    /// * top3/4 in small online brackets (10-20 players)
    /// * default format for kusoges and extremely fast-pace game (example: DBFZ, BBTAG, GGST)
    pub fn ft3() -> Self {
        Self(3)
    }

    /// New match format (you should use `ft2()` and `ft3()`
    fn new(first_to_n: u8) -> Result<Self, MatchFormatError> {
        if first_to_n == 0 {
            Err(MatchFormatError::Invalid(first_to_n))
        } else {
            Ok(Self(first_to_n))
        }
    }
}

/// Creating result
#[derive(Error, Debug)]
pub enum Error {
    /// Match result is invalid
    #[error("Invalid match result {0}")]
    Invalid(Score),
}

impl MatchResult {
    /// New bracket result
    pub fn new(score: Score, format: MatchFormat) -> Result<Self, Error> {
        if format.0 < score.0 || format.0 < score.1 || (format.0 == score.0 && score.0 == score.1) {
            Err(Error::Invalid(score))
        } else {
            Ok(Self(score, format))
        }
    }

    /// Returns true if match is finished
    pub fn finished(&self) -> bool {
        let first_to_n = self.1 .0;
        let player_with_expected_high_seed_score = self.0 .0;
        let player_with_expected_low_seed_score = self.0 .1;

        assert!(player_with_expected_high_seed_score <= first_to_n);
        assert!(player_with_expected_low_seed_score <= first_to_n);

        player_with_expected_high_seed_score == first_to_n
            || player_with_expected_low_seed_score == first_to_n
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}

impl Display for MatchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {}", self.1, self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::matches::result::{Error, MatchFormat, Score};
    use crate::matches::MatchResult;

    #[test]
    fn first_to_two() {
        assert!(MatchResult::new(Score(2, 0), MatchFormat::default()).is_ok());
        assert!(MatchResult::new(Score(2, 1), MatchFormat::default()).is_ok());
        assert!(MatchResult::new(Score(1, 2), MatchFormat::default()).is_ok());
        assert!(MatchResult::new(Score(0, 2), MatchFormat::default()).is_ok());
    }
    #[test]
    fn first_to_three() {
        let format = MatchFormat::ft3();
        assert!(MatchResult::new(Score(3, 0), format).is_ok());
        assert!(MatchResult::new(Score(3, 1), format).is_ok());
        assert!(MatchResult::new(Score(3, 2), format).is_ok());
        assert!(MatchResult::new(Score(2, 3), format).is_ok());
        assert!(MatchResult::new(Score(1, 3), format).is_ok());
        assert!(MatchResult::new(Score(0, 3), format).is_ok());
    }

    #[test]
    fn ft2_is_finished() {
        let format = MatchFormat::ft2();
        let result = MatchResult::new(Score(2, 0), format).unwrap();
        assert!(result.finished());
        let result = MatchResult::new(Score(0, 2), format).unwrap();
        assert!(result.finished());
    }
    #[test]
    fn ft3_is_finished() {
        let format = MatchFormat::ft3();
        let result = MatchResult::new(Score(3, 1), format).unwrap();
        assert!(result.finished());
        let result = MatchResult::new(Score(1, 3), format).unwrap();
        assert!(result.finished());
    }

    #[test]
    fn weird_result_throws_error() {
        let format = MatchFormat::ft2();
        let Err(Error::Invalid(_)) = MatchResult::new(Score(2, 2), format) else {
            panic!()
        };
    }

    #[test]
    fn cannot_report_ft3_final_result_in_ft2_match() {
        let format = MatchFormat::ft2();
        let Err(Error::Invalid(_)) = MatchResult::new(Score(3, 1), format) else {
            panic!()
        };
    }
}
