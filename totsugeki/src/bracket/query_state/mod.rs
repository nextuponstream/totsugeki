//! Query state of bracket

use crate::{
    bracket::{Bracket, Error},
    matches::Id as MatchId,
    opponent::Opponent,
    player::Id as PlayerId,
};

impl Bracket {
    /// Returns true if bracket is over (all matches are played)
    #[must_use]
    pub(super) fn is_over(&self) -> bool {
        self.format
            .get_progression(
                self.get_matches(),
                self.get_participants(),
                self.automatic_match_progression,
            )
            .is_over()
    }

    /// Return next opponent for `player_id`, relevant match and player name
    ///
    /// # Errors
    /// Thrown when matches have yet to be generated or player has won/been
    /// eliminated
    pub fn next_opponent(&self, player_id: PlayerId) -> Result<(Opponent, MatchId, String), Error> {
        match self
            .format
            .get_progression(
                self.get_matches(),
                self.get_participants(),
                self.automatic_match_progression,
            )
            .next_opponent(player_id)
        {
            Ok(el) => Ok(el),
            Err(e) => Err(Error::Progression(self.bracket_id, e)),
        }
    }
}
