use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::progression::ProgressionDEB;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::player::{Participants, Player};
use totsugeki::validation::AutomaticMatchValidationMode;
use totsugeki::ID;

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
        Some(AutomaticMatchValidationMode::Flexible),
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
}
