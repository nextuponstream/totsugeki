//! Query match

use crate::matches::Match;
use crate::opponent::Opponent;
use crate::player::PlayerID;

impl Match {
    /// Loser of the match if any
    pub(crate) fn get_loser(&self) -> Option<PlayerID> {
        match (self.winner, self.players) {
            (Opponent(Some(winner_id)), [Opponent(Some(p1)), Opponent(Some(p2))])
                if winner_id == p1 =>
            {
                Some(p2)
            }
            (Opponent(Some(winner_id)), [Opponent(Some(p1)), Opponent(Some(p2))])
                if winner_id == p2 =>
            {
                Some(p1)
            }
            (Opponent(None), [_, _]) => None,
            _ => unreachable!(),
        }
    }
}
