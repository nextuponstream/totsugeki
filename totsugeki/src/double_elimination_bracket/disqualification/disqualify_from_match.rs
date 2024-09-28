use crate::ID;
use crate::double_elimination_bracket::DoubleEliminationBracket;

impl DoubleEliminationBracket {
    /// Disqualify participant from current match and only current match, allowing player to play
    /// in loser bracket if they were to return.
    ///
    /// Example: player went out and is delaying the bracket unnecessarily
    ///
    /// This is a no-op if the player has played all of their matches
    ///
    /// # Panics
    /// When player_id does not belong in bracket
    pub fn disqualify_participant_from_match(&self, player_id: ID) -> Self {
        todo!()
        // if !self.seeding.contains(player_id) {
        //     panic!("Player does not belong in bracket {player_id:?}")
        // }
    }
}
