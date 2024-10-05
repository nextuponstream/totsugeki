use crate::single_elimination_bracket::{
    assert_next_matches, assert_no_next_match_after_tournament_is_over,
};
use totsugeki::bracket::seeding::Seeding;
use totsugeki::player::{Participants, Player};
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

#[test]
fn bracket_8_man() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let auto = false;

    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), false);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[8].get_id())
        .expect("winner 1vs8");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1], &[(2, 7), (3, 6), (4, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[7].get_id())
        .expect("winner 2vs7");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1, 2], &[(3, 6), (4, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
        .expect("winner 4vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[2], &[(3, 6), (1, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[1].get_id())
        .expect("winner 1vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[2, 5], &[(3, 6)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[3].get_id())
        .expect("winner 3vs6");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[5], &[(2, 6)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[2].get_id())
        .expect("winner 2vs6");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(5, 6)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[6].get_id())
        .expect("winner 5vs6");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);

    assert_no_next_match_after_tournament_is_over(&bracket);
}

#[test]
fn bracket_8_man_manual2() {
    let mut p = vec![Player::new("don't use".into())];
    let mut bad_seeding = Participants::default();
    let mut seeding = vec![];
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
        bad_seeding = bad_seeding.add_participant(player).expect("new player");
    }

    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), false);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[8].get_id())
        .expect("winner 1vs8");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1], &[(2, 7), (3, 6), (4, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[7].get_id())
        .expect("winner 2vs7");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1, 2], &[(3, 6), (4, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
        .expect("winner 4vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[2], &[(3, 6), (1, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[1].get_id())
        .expect("winner 1vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[2, 5], &[(3, 6)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[3].get_id())
        .expect("winner 3vs6");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[5], &[(2, 6)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[2].get_id())
        .expect("winner 2vs6");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(5, 6)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[6].get_id())
        .expect("winner 5vs6");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);

    assert_no_next_match_after_tournament_is_over(&bracket);
}

#[test]
fn bracket_9_man() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=9 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), false);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
        .expect("winner 4vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1, 5], &[(8, 9), (3, 6), (2, 7)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[8].get_id())
        .expect("winner 8vs9");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[5], &[(1, 9), (3, 6), (2, 7)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
        .expect("winner 3vs6");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[3, 5], &[(1, 9), (2, 7)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[7].get_id(), (2, 0), p[2].get_id())
        .expect("winner 3vs6");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[5], &[(1, 9), (3, 7)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[7].get_id())
        .expect("winner 3vs7");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[3, 5], &[(1, 9)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[1].get_id())
        .expect("winner 1vs9");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[3], &[(9, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[5].get_id())
        .expect("winner 5vs9");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(3, 9)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[9].get_id())
        .expect("winner 3vs9");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);

    assert_no_next_match_after_tournament_is_over(&bracket);
}
