//! Disqualification of player in bracket and all side-effects

use crate::{
    bracket::{Bracket, Error},
    matches::Match,
    player::Id as PlayerId,
};

use super::matches::is_disqualified;

/// Returns new matches when comparing old bracket and new bracket
pub(crate) fn get_new_matches(old_bracket: &[Match], new_bracket: &[Match]) -> Vec<Match> {
    new_bracket
        .iter()
        .filter(|new_m| {
            !old_bracket
                .iter()
                .any(|old_m| old_m.get_id() == new_m.get_id())
        })
        .copied()
        .collect::<Vec<Match>>()
}

impl Bracket {
    /// Disqualify player from bracket, advance opponent in bracket and returns
    /// updated bracket
    ///
    /// # Errors
    /// thrown when referred player does not belong in current bracket, bracket
    /// has not started/is over or participant has already been disqualified
    pub fn disqualify_participant(
        self,
        player_id: PlayerId,
    ) -> Result<(Bracket, Vec<Match>), Error> {
        if !self.accept_match_results {
            return Err(Error::NotStarted(
                self.bracket_id,
                ". Cannot disqualify at this time.".into(),
            ));
        }

        let p = self.format.get_progression(
            self.get_matches(),
            &self.get_participants(),
            self.automatic_match_progression,
        );
        let (matches, matches_to_play) = match p.disqualify_participant(player_id) {
            Ok(v) => v,
            Err(e) => return Err(self.get_from_progression_error(e)),
        };
        let bracket = Self { matches, ..self };
        bracket.check_all_assertions();
        Ok((bracket, matches_to_play))
    }

    /// Returns true if player is disqualified
    #[must_use]
    pub fn is_disqualified(&self, player_id: PlayerId) -> bool {
        is_disqualified(player_id, &self.matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::Format;

    #[test]
    fn cannot_disqualify_player_before_bracket_starts() {
        let mut bracket = Bracket {
            format: Format::SingleElimination,
            ..Bracket::default()
        };

        for i in 1..=3 {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("add player");
        }

        let players = bracket.get_participants().get_players_list();
        let p1_id = players[0].get_id();

        let bracket_id = bracket.bracket_id;
        match bracket.disqualify_participant(p1_id) {
            Err(Error::NotStarted(id, _)) => assert_eq!(id, bracket_id),
            Err(e) => panic!("Expected Started error, got {e}"),
            Ok((b, _)) => panic!("Expected error, bracket: {b}"),
        }
    }
}
