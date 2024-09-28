use totsugeki::matches::Match;
use totsugeki::opponent::Opponent;
use totsugeki::player::Player;

/// There is a match with a given `winner` and `loser`
pub fn assert_outcome(matches: &[Match], winner: &Player, loser: &Player) {
    assert!(
        matches.iter().any(|m| matches!((
                m.contains(winner.get_id()),
                m.contains(loser.get_id()),
                m.get_winner()
            ), (true, true, Opponent::Player(match_winner)) if match_winner == winner.get_id())),
        "No match where {} wins against {}",
        winner.get_name(),
        loser.get_name()
    );
}
