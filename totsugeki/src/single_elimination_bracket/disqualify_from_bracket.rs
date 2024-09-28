use crate::opponent::Opponent;
use crate::single_elimination_bracket::progression::ProgressionSEB;
use crate::single_elimination_bracket::SingleEliminationBracket;
use crate::ID;

impl SingleEliminationBracket {
    /// Disqualify participant from bracket completely
    ///
    /// Usually done when the player is unable to attend the bracket at all (missed flight, money
    /// problem...) and warned TO's about it
    pub fn disqualify_participant_from_bracket(self, player_id: ID) -> Self {
        // in the case where all players are disqualified, the last player being disqualified
        // results in a no-op
        if let Some(rev_pos_of_match_with_disqualified_player) = self
            .matches
            .iter()
            .rev()
            .position(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
        {
            let pos = self.matches.len() - 1 - rev_pos_of_match_with_disqualified_player;
            let updated_match = self.matches[pos].set_automatic_loser(player_id).unwrap();
            let mut updated_matches = self.matches;
            updated_matches[pos] = updated_match;
            let b = Self {
                matches: updated_matches,
                ..self
            };
            let (b, _) = b.validate_match_result(updated_match.id);
            b
        } else {
            self
        }
    }
}
