//! next opponent

use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::opponent::Opponent;
use crate::ID;
use thiserror::Error;

#[derive(Error, Debug)]
/// There is no next opponent for player for a given reason
pub enum Error {
    /// Player has won the bracket
    #[error("Player has won bracket")]
    TournamentWon,
    /// Player is eliminated from bracket
    #[error("Player is eliminated from bracket")]
    Eliminated,
}

impl DoubleEliminationBracket {
    // FIXME method next_opponent needs same signature for SingleEliminationBracket struct
    /// Get next opponent of `player_id` and match ID where they have to play
    ///
    /// # Errors
    /// Player will not have next opponent. Returned error enum variant is the
    /// given reason.
    pub fn next_opponent(&self, player_id: ID) -> Result<Option<(Opponent, ID)>, Error> {
        assert!(self.seeding.contains(player_id), "player is not in bracket");
        assert!(!self.matches.is_empty(), "no matches to query");
        if self.is_eliminated(player_id) {
            return Err(Error::Eliminated);
        }

        let next_match = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let Some(relevant_match) = next_match else {
            let last_match = self.matches.iter().last().expect("last match");
            return match last_match.get_winner() {
                Opponent::Player(p) if p == player_id => Err(Error::TournamentWon),
                _ => Err(Error::Eliminated),
            };
        };

        let opponent = match relevant_match.get_players() {
            [Opponent::Player(p1), Opponent::Player(p2)] if p1 == player_id => Opponent::Player(p2),
            [Opponent::Player(p1), Opponent::Player(p2)] if p2 == player_id => Opponent::Player(p1),
            _ => Opponent::Unknown,
        };

        Ok(Some((opponent, relevant_match.get_id())))
    }

    fn is_eliminated(&self, player_id: ID) -> bool {
        let losses = self
            .matches
            .iter()
            .find(|m| m.get_loser() == Some(player_id))
            .iter()
            .count();

        match losses {
            0 | 1 => false,
            2 => true,
            _ => unreachable!(),
        }
    }
}
