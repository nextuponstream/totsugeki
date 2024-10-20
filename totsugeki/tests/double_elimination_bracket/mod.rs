// TODO move all tests of public interface here

use totsugeki::double_elimination_bracket::next_opponent::Error;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::single_elimination_bracket::SingleEliminationBracket;

mod flexible_validation;
mod strict_validation;

fn assert_no_next_match_after_tournament_is_over(bracket: &DoubleEliminationBracket) {
    let mut tournament_winner = 0;
    for player in bracket.get_seeding().get() {
        match bracket.next_opponent(player) {
            Err(Error::Eliminated) => {}
            Err(Error::TournamentWon) => {
                tournament_winner = tournament_winner + 1;
            }
            _ => unreachable!(),
        }
    }

    assert_eq!(tournament_winner, 1)
}
