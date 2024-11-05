//! Upon match validation, bracket progress by moving winners forward and
//! handling loosers

use crate::{matches::Match, opponent::Opponent, player::Id as PlayerId};

/// Get new matches using `old_matches` to play and new matches to play
pub(crate) fn new_matches_to_play_for_bracket(
    old_matches_to_play: &[Match],
    matches_to_play: &[Match],
) -> Vec<Match> {
    assert!(matches_to_play.iter().all(|m| m.needs_playing()));
    assert!(
        old_matches_to_play.iter().all(|m| m.needs_playing()),
        "{:?}",
        old_matches_to_play
    );
    let new_matches_to_play: Vec<Match> = matches_to_play
        .iter()
        .filter(|m| {
            old_matches_to_play
                .iter()
                .all(|old_m| old_m.get_id() != m.get_id())
        })
        .map(Clone::clone)
        .collect();

    if new_matches_to_play.len() > 2 {
        panic!(
            "Misuse: when resolving in a bracket, the winner of the match goes to his next match \
        and same thing for the loser. Therefore, there should be at most two new matches to play \
         but found ({})",
            new_matches_to_play.len()
        )
    }
    new_matches_to_play
}

/// Returns winner of bracket
pub(crate) fn winner_of_bracket(bracket: &[Match]) -> Option<PlayerId> {
    match bracket.last() {
        Some(m) => match m.get_winner() {
            Opponent::Player(p) => Some(p),
            Opponent::Unknown => None,
        },
        None => None,
    }
}
