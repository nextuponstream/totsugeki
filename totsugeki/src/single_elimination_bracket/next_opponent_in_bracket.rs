use crate::next_opponent::NextOpponentInBracket;
use crate::opponent::Opponent;
use crate::single_elimination_bracket::SingleEliminationBracket;
use crate::ID;

impl NextOpponentInBracket for SingleEliminationBracket {
    fn next_opponent_in_bracket(&self, player_id: ID) -> Option<(Option<Opponent>, ID)> {
        todo!()
    }
}
