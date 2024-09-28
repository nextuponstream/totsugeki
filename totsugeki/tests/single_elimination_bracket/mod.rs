mod disqualify_from_bracket;

use totsugeki::bracket::seeding::Seeding;
use totsugeki::player::{Participants, Player};
use totsugeki::single_elimination_bracket::SingleEliminationBracket;

#[test]
fn run_5_man_bracket() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=5 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let seeding = Seeding::new(seeding).unwrap();
    let auto = false;
    let s = SingleEliminationBracket::create(seeding, auto);

    todo!()
    // let (seb, match_id, _) = s
    //     .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
    //     .expect("winner 4vs5");
    // let (seb, new_matches) = s.validate_match_result(match_id);
    // assert_eq!(new_matches.len(), 1, "{new_matches:?}");
    // assert_next_matches(seb, &[], &[(1, 5), (2, 3)], &p);
    //
    // let (matches, match_id, _) = s
    //     .tournament_organiser_reports_result(p[1].get_id(), (2, 1), p[5].get_id())
    //     .expect("winner 1vs5");
    // let s = Step::new(matches, &s.seeding, auto);
    // let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
    // let s = Step::new(matches, &s.seeding, auto);
    // assert_eq!(new_matches.len(), 0);
    // assert_next_matches(&s, &[1], &[(2, 3)], &p);
    //
    // let (matches, match_id, _) = s
    //     .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[2].get_id())
    //     .expect("winner 2vs3");
    // let s = Step::new(matches, &s.seeding, auto);
    // let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
    // assert_eq!(new_matches.len(), 1);
    // let s = Step::new(matches, &s.seeding, auto);
    // assert_next_matches(&s, &[], &[(1, 3)], &p);
    //
    // let (matches, match_id, _) = s
    //     .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[1].get_id())
    //     .expect("winner 1vs3");
    // let s = Step::new(matches, &s.seeding, auto);
    // let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
    // assert_eq!(new_matches.len(), 0);
    //
    // let s = Step::new(matches, &s.seeding, auto);
    // assert_elimination(&s, &p, 3);
}
