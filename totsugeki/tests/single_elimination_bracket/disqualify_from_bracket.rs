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
    todo!()
    // let (matches, _) = s
    //     .disqualify_participant(p[4].get_id())
    //     .expect("bracket with player 4 disqualified");
    // let s = Step::new(matches, &s.seeding, auto);
    // assert_outcome(&s.matches, &p[5], &p[4]);
    // let (matches, _) = s
    //     .disqualify_participant(p[5].get_id())
    //     .expect("bracket with player 5 disqualified");
    // let s = Step::new(matches, &s.seeding, auto);
    // // player 5 opponent is unknown
    // let (matches, _) = s
    //     .disqualify_participant(p[6].get_id())
    //     .expect("bracket with player 6 disqualified");
    // let s = Step::new(matches, &s.seeding, auto);
    // assert_outcome(&s.matches, &p[7], &p[6]);
    // let (matches, _) = s
    //     .disqualify_participant(p[7].get_id())
    //     .expect("bracket with player 7 disqualified");
    // let s = Step::new(matches, &s.seeding, auto);
    // // player 7 is in GF
    // let (matches, _) = s
    //     .disqualify_participant(p[8].get_id())
    //     .expect("bracket with player 8 disqualified");
    // let s = Step::new(matches, &s.seeding, auto);
    // assert_outcome(&s.matches, &p[1], &p[8]);
    // assert_outcome(&s.matches, &p[1], &p[5]);
    // assert_outcome(&s.matches, &p[1], &p[7]);
    //
    // match s.disqualify_participant(p[1].get_id()) {
    //     Err(Error::TournamentIsOver) => {}
    //     Err(e) => panic!("Expected Tournament over error but got {e:?}"),
    //     Ok(_) => panic!("Expected error but none returned: {s:?}"),
    // };
}
