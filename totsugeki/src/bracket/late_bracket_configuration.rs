//! For spectacle, match format changes from FT2 to FT3 (maximize screen time
//! of the best tournament players). However, depending on the game,
//! participation and tournament venue, the format change may occur only
//! very late in the bracket

use crate::matches::result::MatchFormat;

/// Late stage bracket configuration
///
/// Most of the bracket is played in a match format. When configured, use
/// a different match format for the last matches
#[derive(Copy, Clone)]
pub struct LateBracketConfiguration {
    /// Use match format for the last `rounds`
    ///
    /// Rounds affect the last 2^n matches of the bracket.
    ///
    /// Examples:
    /// * the winner bracket of a double elimination bracket is played in FT3
    ///   only for the winner finals. Then `rounds` is configured to 1.
    /// * the winner bracket of a double elimination bracket top 8 is in FT3.
    ///   Then `rounds` is set to 3
    #[allow(dead_code)]
    rounds: usize,
    /// For the last `rounds`, use `match_format` instead
    #[allow(dead_code)]
    match_format: MatchFormat,
}
