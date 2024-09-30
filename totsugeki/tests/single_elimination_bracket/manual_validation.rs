use crate::single_elimination_bracket::{
    assert_next_matches, assert_no_next_match_after_tournament_is_over,
};
use totsugeki::bracket::seeding::Seeding;
use totsugeki::player::Player;
use totsugeki::single_elimination_bracket::progression::ProgressionSEB;
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
    let bracket = SingleEliminationBracket::create(seeding, false);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
        .unwrap();
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(
        new_matches.len(),
        1,
        "{new_matches:?}\nmatches {:?}",
        bracket.get_matches()
    );
    assert_next_matches(&bracket, &[], &[(1, 5), (2, 3)], &p);
    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[1].get_id(), (2, 1), p[5].get_id())
        .expect("winner 1vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1], &[(2, 3)], &p);
    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[2].get_id())
        .expect("winner 2vs3");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(1, 3)], &p);
    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[1].get_id())
        .expect("winner 1vs3");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);

    assert_no_next_match_after_tournament_is_over(&bracket);
}
