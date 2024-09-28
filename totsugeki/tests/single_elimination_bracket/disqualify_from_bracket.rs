use crate::common;
use crate::common::assert_outcome;
use totsugeki::bracket::seeding::Seeding;
use totsugeki::player::Player;
use totsugeki::single_elimination_bracket::SingleEliminationBracket;

#[test]
fn disqualifying_everyone_is_impossible_because_the_last_player_remaining_wins_grand_finals_automatically(
) {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let seeding = Seeding::new(seeding).unwrap();
    let auto = false;
    let bracket = SingleEliminationBracket::create(seeding, auto);
    let bracket = bracket.disqualify_participant_from_bracket(p[2].get_id());
    assert_outcome(&bracket.get_matches(), &p[7], &p[2]);
    let bracket = bracket.disqualify_participant_from_bracket(p[3].get_id());
    assert_outcome(&bracket.get_matches(), &p[6], &p[3]);
    let bracket = bracket.disqualify_participant_from_bracket(p[4].get_id());
    assert_outcome(&bracket.get_matches(), &p[5], &p[4]);
    let bracket = bracket.disqualify_participant_from_bracket(p[5].get_id());
    // player 5 opponent is unknown
    let bracket = bracket.disqualify_participant_from_bracket(p[6].get_id());
    assert_outcome(&bracket.get_matches(), &p[7], &p[6]);
    let bracket = bracket.disqualify_participant_from_bracket(p[7].get_id());
    // player 7 is in GF
    let bracket = bracket.disqualify_participant_from_bracket(p[8].get_id());
    assert_outcome(&bracket.get_matches(), &p[1], &p[8]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[5]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[7]);

    let bracket = bracket.disqualify_participant_from_bracket(p[1].get_id());
    assert_outcome(&bracket.get_matches(), &p[1], &p[8]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[5]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[7]);
}
