//! next opponent

use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::MatchID;
use crate::next_opponent::{Error, NextOpponentInBracket};
use crate::opponent::Opponent;
use crate::player::PlayerID;

impl DoubleEliminationBracket {
    /// Returns `true` if player is eliminated from bracket
    #[must_use]
    pub fn is_eliminated(&self, player_id: PlayerID) -> bool {
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

impl NextOpponentInBracket for DoubleEliminationBracket {
    fn next_opponent_in_bracket(&self, player_id: PlayerID) -> Result<(Opponent, MatchID), Error> {
        assert!(self.seeding.contains(player_id), "player is not in bracket");
        assert!(!self.matches.is_empty(), "no matches to query");
        if self.is_eliminated(player_id) {
            return Err(Error::Eliminated);
        }

        // did they win grand final as the highest seed? That means tournament
        // is over
        let grand_final = self.matches[self.matches.len() - 2];
        if let Opponent(Some(winner)) = grand_final.winner {
            if let Opponent(Some(higher_seed)) = grand_final.players[0] {
                if player_id == higher_seed && higher_seed == winner {
                    return Err(Error::TournamentWon);
                }
            }
        }

        // Let's look at the next match if any
        let next_match = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent(None));
        let Some(relevant_match) = next_match else {
            // No next match? Did they win through bracket reset?
            let last_match = self.matches.iter().last().expect("last match");
            return match last_match.get_winner() {
                Opponent(Some(p)) if p == player_id => Err(Error::TournamentWon),
                _ => Err(Error::Eliminated),
            };
        };

        // Determine opponent by taking the other player
        let opponent = match relevant_match.get_players() {
            [Opponent(Some(p1)), Opponent(Some(p2))] if p1 == player_id => Opponent(Some(p2)),
            [Opponent(Some(p1)), Opponent(Some(p2))] if p2 == player_id => Opponent(Some(p1)),
            _ => Opponent(None),
        };

        Ok((opponent, relevant_match.get_id()))
    }
}
