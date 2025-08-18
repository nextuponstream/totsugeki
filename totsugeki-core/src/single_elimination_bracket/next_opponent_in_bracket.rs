//! Inspect next opponent for a given player in single elimination bracket

use crate::next_opponent::{Error, NextOpponentInBracket};
use crate::opponent::Opponent;
use crate::single_elimination_bracket::SingleEliminationBracket;
use crate::ID;

impl NextOpponentInBracket for SingleEliminationBracket {
    fn next_opponent_in_bracket(&self, player_id: &ID) -> Result<(Opponent, ID), Error> {
        if self.is_over() {
            let Opponent(Some(winner)) = self.matches.last().expect("matches").winner else {
                unreachable!("tournament is over but without a winner")
            };
            return if player_id == &winner {
                Err(Error::TournamentWon)
            } else {
                Err(Error::Eliminated)
            };
        }
        let next_match = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && *m.get_winner() == Opponent(None));
        let Some(relevant_match) = next_match else {
            return Err(Error::Eliminated);
        };

        let opponent = match &relevant_match.get_players() {
            [Opponent(Some(p1)), Opponent(Some(p2))] if p1 == player_id => Opponent(Some(*p2)),
            [Opponent(Some(p1)), Opponent(Some(p2))] if p2 == player_id => Opponent(Some(*p1)),
            _ => Opponent(None),
        };
        Ok((opponent, *relevant_match.get_id()))
    }
}
