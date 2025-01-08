// TODO move all tests of public interface here

use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::next_opponent::Error;
use totsugeki_core::next_opponent::NextOpponentInBracket;

pub mod disqualify_from_bracket;
mod flexible_validation;
mod reporting;
mod strict_validation;

fn assert_no_next_match_after_tournament_is_over(bracket: &DoubleEliminationBracket) {
    let mut tournament_winner = 0;
    for player in bracket.get_seeding().get() {
        match bracket.next_opponent_in_bracket(player) {
            Err(Error::Eliminated) => {}
            Err(Error::TournamentWon) => {
                tournament_winner = tournament_winner + 1;
            }
            _ => unreachable!(),
        }
    }

    assert_eq!(tournament_winner, 1, "{:?}", bracket.get_matches())
}
