use crate::double_elimination_bracket::assert_no_next_match_after_tournament_is_over;
use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::progression::ProgressionDEB;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::player::{Participants, Player};
use totsugeki::validation::AutomaticMatchValidationMode;
use totsugeki::ID;

#[test]
fn bracket_run_3_man() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = Participants::default();
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding = seeding.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_seeding()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
    );

    assert_eq!(bracket.get_matches().len(), 5);
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (2, 0), p[3].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (0, 2), p[2].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (0, 2), p[3].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (0, 2), p[3].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (0, 2), p[3].get_id())
        .expect("s");
    assert!(bracket.is_over());
    assert_no_next_match_after_tournament_is_over(&bracket);
}

#[test]
fn run_3_man_bracket() {
    let mut player_ids = vec![ID::new_v4()]; // padding for readability
    let mut unpadded_player_ids = vec![];
    let mut seeding = Participants::default();
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        player_ids.push(player.get_id());
        unpadded_player_ids.push(player.get_id());
        seeding = seeding.add_participant(player).expect("new participant");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(unpadded_player_ids.clone()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
    );

    assert_eq!(bracket.get_matches().len(), 5);
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (2, 0), player_ids[3])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (0, 2), player_ids[2])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (0, 2), player_ids[3])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (0, 2), player_ids[3])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (0, 2), player_ids[3])
        .expect("bracket");
    assert!(bracket.is_over());
    assert_no_next_match_after_tournament_is_over(&bracket);
}
#[test]
fn run_5_man_bracket() {
    let mut player_ids = vec![ID::new_v4()]; // padding for readability
    let mut seeding = Participants::default();
    for i in 1..=5 {
        let player = Player::new(format!("p{i}"));
        player_ids.push(player.get_id());
        seeding = seeding.add_participant(player).expect("new participant");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_seeding()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
    );

    assert_eq!(bracket.get_matches().len(), 9);
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (0, 2), player_ids[3])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[4], (0, 2), player_ids[5])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (2, 0), player_ids[5])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (0, 2), player_ids[3])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[5], (2, 0), player_ids[4])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (2, 0), player_ids[5])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (2, 0), player_ids[1])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (0, 2), player_ids[3])
        .expect("bracket");
    assert!(bracket.is_over());
    assert_no_next_match_after_tournament_is_over(&bracket);
}

#[test]
fn run_5_man() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = Participants::default();
    for i in 1..=5 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding = seeding.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_seeding()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
    );

    assert_eq!(bracket.get_matches().len(), 9);
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (0, 2), p[3].get_id())
        .expect("step");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[4].get_id(), (0, 2), p[5].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (2, 0), p[5].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (0, 2), p[3].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[5].get_id(), (2, 0), p[4].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (2, 0), p[5].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (2, 0), p[1].get_id())
        .expect("s");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (0, 2), p[3].get_id())
        .expect("s");
    assert!(bracket.is_over());
    assert_no_next_match_after_tournament_is_over(&bracket);
}
#[test]
fn run_8_man_bracket_no_upsets() {
    let mut player_ids = vec![ID::new_v4()]; // padding for readability
    let mut unpadded_player_ids = vec![];
    let mut seeding = Participants::default();
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        player_ids.push(player.get_id());
        unpadded_player_ids.push(player.get_id());
        seeding = seeding.add_participant(player).expect("new participant");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(unpadded_player_ids).unwrap(),
        AutomaticMatchValidationMode::Flexible,
    );

    assert_eq!(bracket.get_matches().len(), 15);
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (2, 0), player_ids[8])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (2, 0), player_ids[7])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[3], (2, 0), player_ids[6])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[4], (2, 0), player_ids[5])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[5], (2, 0), player_ids[8])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[6], (2, 0), player_ids[7])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (2, 0), player_ids[4])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (2, 0), player_ids[3])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[3], (2, 0), player_ids[6])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[4], (2, 0), player_ids[5])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[3], (2, 0), player_ids[4])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (2, 0), player_ids[2])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (2, 0), player_ids[3])
        .expect("bracket");
    let (bracket, _, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (2, 0), player_ids[2])
        .expect("bracket");
    assert!(bracket.is_over());
    assert_no_next_match_after_tournament_is_over(&bracket);
}
