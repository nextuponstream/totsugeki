use crate::single_elimination_bracket::{
    assert_next_matches, assert_no_next_match_after_tournament_is_over,
};
use totsugeki::bracket::seeding::Seeding;
use totsugeki::next_opponent::NextOpponentInBracket;
use totsugeki::opponent::Opponent;
use totsugeki::player::Player;
use totsugeki::single_elimination_bracket::progression::ProgressionSEB;
use totsugeki::single_elimination_bracket::SingleEliminationBracket;

fn assert_players_play_each_other(
    player_1: usize,
    player_2: usize,
    player_ids: &[Player],
    bracket: &SingleEliminationBracket,
) {
    let Some((Some(Opponent::Player(next_opponent)), match_id_1)) =
        bracket.next_opponent_in_bracket(player_ids[player_1].get_id())
    else {
        panic!("No next opponent")
    };
    assert_eq!(next_opponent, player_ids[player_2].get_id());

    let Some((Some(Opponent::Player(next_opponent)), match_id_2)) =
        bracket.next_opponent_in_bracket(player_ids[player_2].get_id())
    else {
        panic!("No next opponent")
    };
    assert_eq!(next_opponent, player_ids[player_1].get_id());

    assert_eq!(
        match_id_1, match_id_2,
        "expected player to be playing the same match"
    );
}

#[test]
fn run_3_man() {
    let mut p = vec![Player::new("don't use".into())]; // padding for readability
    let mut seeding = vec![];
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let seb = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

    assert_eq!(seb.get_matches().len(), 2);
    assert_eq!(seb.matches_to_play().len(), 1);
    assert_players_play_each_other(2, 3, &p, &seb);
    let (bracket, _, new_matches) = seb
        .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
        .expect("bracket");
    assert_eq!(new_matches.len(), 1, "grand finals match generated");
    assert_players_play_each_other(1, 2, &p, &bracket);
    assert_eq!(bracket.matches_to_play().len(), 1);
    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[2].get_id())
        .expect("bracket");
    assert!(bracket.matches_to_play().is_empty());
    assert!(new_matches.is_empty());
    assert!(bracket.is_over());
}

#[test]
fn run_5_man_automated_2() {
    let mut p = vec![Player::new("don't use".into())]; // padding for readability
    let mut seeding = vec![];
    for i in 1..=5 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

    assert_eq!(bracket.get_matches().len(), 4);
    assert_eq!(bracket.matches_to_play().len(), 2);
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result(p[4].get_id(), (2, 0), p[5].get_id())
        .expect("bracket");
    assert_eq!(bracket.matches_to_play().len(), 2);
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[3].get_id())
        .expect("bracket");
    assert_eq!(bracket.matches_to_play().len(), 1);
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[4].get_id())
        .expect("bracket");
    assert_eq!(bracket.matches_to_play().len(), 1);
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[3].get_id())
        .expect("bracket");
    assert!(bracket.is_over());
    assert_eq!(bracket.matches_to_play().len(), 0);
}

#[test]
fn run_5_man_bracket_automated() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=5 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
        .expect("winner 4vs5");
    assert_eq!(new_matches.len(), 1, "{new_matches:?}");
    assert_next_matches(&bracket, &[], &[(1, 5), (2, 3)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[1].get_id(), (2, 1), p[5].get_id())
        .expect("winner 1vs5");
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1], &[(2, 3)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[2].get_id())
        .expect("winner 2vs3");
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(1, 3)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[1].get_id())
        .expect("winner 1vs3");
    assert_eq!(new_matches.len(), 0);

    assert_no_next_match_after_tournament_is_over(&bracket);
}

#[test]
fn bracket_8_man_automated() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[8].get_id())
        .expect("winner 1vs8");
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1], &[(2, 7), (3, 6), (4, 5)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[7].get_id())
        .expect("winner 2vs7");
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1, 2], &[(3, 6), (4, 5)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
        .expect("winner 4vs5");
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[2], &[(3, 6), (1, 5)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[1].get_id())
        .expect("winner 1vs5");
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[2, 5], &[(3, 6)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[3].get_id())
        .expect("winner 3vs6");
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[5], &[(2, 6)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[2].get_id())
        .expect("winner 2vs6");
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(5, 6)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[6].get_id())
        .expect("winner 5vs6");
    assert_eq!(new_matches.len(), 0);

    assert_no_next_match_after_tournament_is_over(&bracket);
}

#[test]
fn bracket_9_man_automated() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=9 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
        .expect("winner 4vs5");
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[1, 5], &[(8, 9), (3, 6), (2, 7)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[8].get_id())
        .expect("winner 8vs9");
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[5], &[(1, 9), (3, 6), (2, 7)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
        .expect("winner 3vs6");
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[3, 5], &[(1, 9), (2, 7)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[7].get_id(), (2, 0), p[2].get_id())
        .expect("winner 3vs6");
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[5], &[(1, 9), (3, 7)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[7].get_id())
        .expect("winner 3vs7");
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[3, 5], &[(1, 9)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[1].get_id())
        .expect("winner 1vs9");
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[3], &[(9, 5)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[5].get_id())
        .expect("winner 5vs9");
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(3, 9)], &p);

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[9].get_id())
        .expect("winner 3vs9");
    assert_eq!(new_matches.len(), 0);

    assert_no_next_match_after_tournament_is_over(&bracket);
}
