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
    pub fn is_over(&self) -> bool {
        self.format
            .get_progression(
                self.get_matches(),
                &self.get_participants(),
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
                &self.get_participants(),
                self.automatic_match_progression,
            )
            .next_opponent(player_id)
        {
            Ok((opponent, match_id)) => Ok((
                opponent,
                match_id,
                self.participants
                    .get(player_id)
                    .expect("player")
                    .to_string(),
            )),
            Err(e) => Err(self.get_from_progression_error(e)),
        }
    }
}
