//! Disqualify player with no chance to play again.

use crate::matches::Match;
use crate::opponent::Opponent;
use crate::single_elimination_bracket::SingleEliminationBracket;
use crate::ID;

impl SingleEliminationBracket {
    /// Disqualify participant from bracket completely. Returns updated bracket and new playable
    /// matches if any
    ///
    /// Usually done when the player is unable to attend the bracket at all (missed flight, money
    /// problem...) and warned TO's about it
    #[must_use]
    pub fn disqualify_participant_from_bracket(
        self,
        player_id: &ID,
    ) -> (SingleEliminationBracket, Option<Vec<Match>>) {
        // in the case where all players are disqualified, the last player being disqualified
        // results in a no-op
        let old_playable_matches = self.matches_to_play();
        if let Some(rev_pos_of_match_with_disqualified_player) = self
            .matches
            .iter()
            .rev()
            .position(|m| m.contains(player_id) && *m.get_winner() == Opponent(None))
        {
            let pos = self.matches.len() - 1 - rev_pos_of_match_with_disqualified_player;
            let updated_match = self.matches[pos].set_automatic_loser(player_id);
            let mut updated_matches = self.matches;
            updated_matches[pos] = updated_match;
            let b = Self {
                matches: updated_matches,
                ..self
            };
            let (b, _) = b.validate_match_result(&updated_match.id);
            let new_matches_to_play = b.matches_to_play();
            let new_playable_matches = new_matches_to_play
                .into_iter()
                .filter(|new| old_playable_matches.iter().any(|old| old.id == new.id))
                .collect::<Vec<Match>>();
            if new_playable_matches.is_empty() {
                (b, None)
            } else {
                (b, Some(new_playable_matches))
            }
        } else {
            (self, None)
        }
    }
}
