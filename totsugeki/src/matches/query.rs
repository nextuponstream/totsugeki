//! Query match

use crate::matches::{Match, MatchPlayers};
use crate::opponent::Opponent;
use crate::ID;

impl Match {
    /// Loser of the match if any
    pub(crate) fn get_loser(&self) -> Option<ID> {
        match (self.winner, self.players) {
            (Opponent::Player(winner_id), [Opponent::Player(p1), Opponent::Player(p2)])
                if winner_id == p1 =>
            {
                Some(p2)
            }
            (Opponent::Player(winner_id), [Opponent::Player(p1), Opponent::Player(p2)])
                if winner_id == p2 =>
            {
                Some(p1)
            }
            (Opponent::Unknown, [_, _]) => None,
            _ => unreachable!(),
        }
    }
}
