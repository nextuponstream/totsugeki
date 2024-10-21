//! Upon match validation, bracket progress by moving winners forward and
//! handling loosers

use super::{Bracket, Error};
use crate::{
    matches::{Id as MatchId, Match},
    opponent::Opponent,
    player::Id as PlayerId,
};

impl Bracket {
    /// Validate match result and return updated bracket with new matches.
    /// Winner moves forward in bracket. If final match is validated, then
    /// bracket will stop accepting match result.
    ///
    /// # Errors
    /// Thrown when given match id is unknown or when reported results differ
    pub fn validate_match_result(self, match_id: MatchId) -> Result<(Self, Vec<Match>), Error> {
        let p = self.format.get_progression(
            self.matches.clone(),
            &self.get_participants(),
            self.automatic_match_progression,
        );
        let (matches, new_matches) = match p.validate_match_result(match_id) {
            Ok(el) => el,
            Err(e) => return Err(self.get_from_progression_error(e)),
        };

        let bracket = Self { matches, ..self };
        let bracket = Self {
            accept_match_results: !bracket.is_over(),
            ..bracket
        };

        Ok((bracket, new_matches))
    }
}

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

#[cfg(test)]
mod tests {

    use crate::{
        bracket::Bracket,
        format::Format,
        matches::partition_double_elimination_matches,
        opponent::Opponent,
        player::{Id as PlayerId, Player},
        seeding::Method,
    };
    use chrono::prelude::*;

    fn assert_players_play_each_other(
        player_1: usize,
        player_2: usize,
        player_ids: &[PlayerId],
        bracket: &Bracket,
    ) {
        let (next_opponent, match_id_1, _msg) = bracket
            .next_opponent(player_ids[player_1])
            .expect("next opponent");
        let Opponent::Player(next_opponent) = next_opponent else {
            panic!("expected player")
        };
        assert_eq!(next_opponent, player_ids[player_2]);

        let (next_opponent, match_id_2, _msg) = bracket
            .next_opponent(player_ids[player_2])
            .expect("next opponent");
        let Opponent::Player(next_opponent) = next_opponent else {
            panic!("expected player")
        };
        assert_eq!(next_opponent, player_ids[player_1]);

        assert_eq!(
            match_id_1, match_id_2,
            "expected player to be playing the same match"
        );
    }

    #[test]
    fn partition_matches_for_3_man_bracket() {
        let mut bracket = Bracket::new(
            "",
            Format::DoubleElimination,
            Method::Strict,
            Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
            true,
        );
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            bracket = bracket.join(player).expect("bracket");
        }

        let (winner_bracket, loser_bracket, _gf, _gfr) = partition_double_elimination_matches(
            &bracket.get_matches(),
            bracket.get_participants().len(),
        );
        assert_eq!(winner_bracket.len(), 2);
        assert_eq!(loser_bracket.len(), 1);
        assert_eq!(loser_bracket[0].get_seeds(), [2, 3]);
    }
}
