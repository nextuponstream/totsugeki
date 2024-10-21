use crate::next_opponent::NextOpponentInBracket;
use crate::opponent::Opponent;
use crate::single_elimination_bracket::SingleEliminationBracket;
use crate::ID;

impl NextOpponentInBracket for SingleEliminationBracket {
    fn next_opponent_in_bracket(&self, player_id: ID) -> Option<(Option<Opponent>, ID)> {
        let next_match = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let Some(relevant_match) = next_match else {
            return None;
        };

        let opponent = match &relevant_match.get_players() {
            [Opponent::Player(p1), Opponent::Player(p2)] if *p1 == player_id => {
                Opponent::Player(*p2)
            }
            [Opponent::Player(p1), Opponent::Player(p2)] if *p2 == player_id => {
                Opponent::Player(*p1)
            }
            _ => Opponent::Unknown,
        };
        Some((Some(opponent), relevant_match.get_id()))
    }
}
