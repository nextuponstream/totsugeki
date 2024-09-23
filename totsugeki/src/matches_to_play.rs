use crate::matches::Match;

/// Query matches to play through consistent interface
pub trait MatchesToPlay {
    /// Return all matches to be played
    fn matches_to_play(&self) -> Vec<Match>;
}
