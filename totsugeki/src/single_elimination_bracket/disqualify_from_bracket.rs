use crate::opponent::Opponent;
use crate::single_elimination_bracket::SingleEliminationBracket;
use crate::ID;

impl SingleEliminationBracket {
    /// Disqualify participant from bracket completely
    ///
    /// Usually done when the player is unable to attend the bracket at all (missed flight, money
    /// problem...) and warned TO's about it
    pub fn disqualify_participant_from_bracket(self, player_id: ID) -> Self {
        if let Some(pos_of_match_with_disqualified_player) = self
            .matches
            .iter()
            .rev()
            .position(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
        {
            let updated_match = self.matches[pos_of_match_with_disqualified_player]
                .set_automatic_loser(player_id)
                .unwrap();
            let mut updated_matches = self.matches;
            updated_matches[pos_of_match_with_disqualified_player] = updated_match;
            Self {
                matches: updated_matches,
                ..self
            }
        } else {
            self
        }
    }
}
