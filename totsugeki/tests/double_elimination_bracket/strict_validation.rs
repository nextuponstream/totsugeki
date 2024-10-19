use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::progression::ProgressionDEB;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::player::{Participants, Player};
use totsugeki::validation::AutomaticMatchValidationMode;
use totsugeki::ID;

fn report(
    double_elimination_bracket: DoubleEliminationBracket,
    player1: ID,
    result: (i8, i8),
    player2: ID,
) -> (DoubleEliminationBracket, ID) {
    let (bracket, m_id, _new_matches) = double_elimination_bracket
        .tournament_organiser_reports_result_dangerous(player1, result, player2)
        .expect("bracket");
    (bracket, m_id)
}

#[test]
fn run_8_man_bracket_with_frequent_upsets() {
    // every 2 matches, there is an upset
    let mut player_ids = vec![ID::new_v4()]; // padding for readability
    let mut unpadded_player_ids = vec![]; // padding for readability
    let mut seeding = Participants::default();
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        player_ids.push(player.get_id());
        unpadded_player_ids.push(player.get_id());
        seeding = seeding.add_participant(player).expect("new participant");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(unpadded_player_ids).unwrap(),
        AutomaticMatchValidationMode::Strict,
    );
    assert_eq!(bracket.get_matches().len(), 15);
    let (bracket, winner_1vs8) = report(bracket, player_ids[1], (2, 0), player_ids[8]);
    let bracket = bracket.validate_match_result(winner_1vs8).0;
    let (bracket, winner_2vs7) = report(bracket, player_ids[2], (0, 2), player_ids[7]);
    let bracket = bracket.validate_match_result(winner_2vs7).0;
    let (bracket, winner_3vs6) = report(bracket, player_ids[3], (2, 0), player_ids[6]);
    let bracket = bracket.validate_match_result(winner_3vs6).0;
    let (bracket, winner_4vs5) = report(bracket, player_ids[4], (0, 2), player_ids[5]);
    let bracket = bracket.validate_match_result(winner_4vs5).0;
    let (bracket, loser_4vs8) = report(bracket, player_ids[4], (2, 0), player_ids[8]);
    let bracket = bracket.validate_match_result(loser_4vs8).0;
    let (bracket, loser_2vs6, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[2], (0, 2), player_ids[6])
        .expect("bracket");
    let bracket = bracket.validate_match_result(loser_2vs6).0;
    let (bracket, winner_1vs5, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (2, 0), player_ids[5])
        .expect("bracket");
    let bracket = bracket.validate_match_result(winner_1vs5).0;
    let (bracket, winner_3vs7, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[3], (0, 2), player_ids[7])
        .expect("bracket");
    let bracket = bracket.validate_match_result(winner_3vs7).0;
    let (bracket, loser_3vs6, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[3], (2, 0), player_ids[6])
        .expect("bracket");
    let bracket = bracket.validate_match_result(loser_3vs6).0;
    let (bracket, loser_4vs5, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[4], (0, 2), player_ids[5])
        .expect("bracket");
    let bracket = bracket.validate_match_result(loser_4vs5).0;
    let (bracket, loser_3vs5, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[3], (2, 0), player_ids[5])
        .expect("bracket");
    let bracket = bracket.validate_match_result(loser_3vs5).0;
    let (bracket, winner_1vs7, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (0, 2), player_ids[7])
        .expect("bracket");
    let bracket = bracket.validate_match_result(winner_1vs7).0;
    let (bracket, loser_1vs3, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (2, 0), player_ids[3])
        .expect("bracket");
    let bracket = bracket.validate_match_result(loser_1vs3).0;
    let (bracket, grand_finals, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(player_ids[1], (0, 2), player_ids[7])
        .expect("bracket");
    let bracket = bracket.validate_match_result(grand_finals).0;
    assert!(bracket.is_over(), "{bracket:?}");
}
