use totsugeki::bracket::matches::{Error, Progression};
use totsugeki::single_elimination_bracket::progression::ProgressionSEB;
mod automatic_validation;
mod disqualify_from_bracket;
mod manual_validation;

use totsugeki::bracket::seeding::Seeding;
use totsugeki::opponent::Opponent;
use totsugeki::player::{Participants, Player};
use totsugeki::single_elimination_bracket::SingleEliminationBracket;
use totsugeki::ID;

fn assert_next_matches(
    bracket: &SingleEliminationBracket,
    players_with_unknown_opponent: &[usize],
    expected_matches: &[(usize, usize)],
    players: &[Player],
) {
    for p in players_with_unknown_opponent {
        let player = players[*p].clone();
        let (next_opponent, _) = bracket
            .next_opponent(player.get_id())
            .expect("next opponent");
        assert_eq!(
            next_opponent,
            Opponent::Unknown,
            "expected unknown opponent for {p} but got {next_opponent}"
        );
    }

    for (o1, o2) in expected_matches {
        let opponent1 = players[*o1].clone();
        let opponent2 = players[*o2].clone();

        let (next_opponent, _) = bracket
            .next_opponent(opponent1.get_id())
            .expect("next opponent");
        let Opponent::Player(p) = next_opponent else {
            panic!("expected player for next opponent");
        };
        assert_eq!(
            p,
            opponent2.get_id(),
            "expected {opponent2} for {opponent1} but got {p}"
        );
        let (next_opponent, _) = bracket
            .next_opponent(opponent2.get_id())
            .expect("next opponent");
        let Opponent::Player(p) = next_opponent else {
            panic!("expected player for next opponent");
        };
        assert_eq!(
            p,
            opponent1.get_id(),
            "expected {opponent1} for {opponent2} but got {p}"
        );
    }
}

fn assert_no_next_match_after_tournament_is_over(bracket: &SingleEliminationBracket) {
    for player in bracket.get_seeding().get().iter() {
        if let Some(next_opponent) = bracket.next_opponent(*player) {
            panic!("expected no next match when tournament is over but got {next_opponent:?}",)
        }
    }
}
