use crate::double_elimination_bracket::assert_no_next_match_after_tournament_is_over;
use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::progression::ProgressionDEB;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::opponent::Opponent;
use totsugeki::player::{Participants, Player};
use totsugeki::validation::AutomaticMatchValidationMode;
use totsugeki::ID;

#[cfg(test)]
pub(crate) fn assert_next_matches(
    bracket: &DoubleEliminationBracket,
    players_with_unknown_opponent: &[usize],
    expected_matches: &[(usize, usize)],
    players: &[Player],
) {
    for p in players_with_unknown_opponent {
        let player = players[*p].clone();
        let (next_opponent, _) = bracket
            .next_opponent(player.get_id())
            .expect("next opponent")
            .expect("opponent is not missing");
        assert_eq!(
            next_opponent,
            Opponent::Unknown,
            "expected unknown opponent for {p} but got {next_opponent}"
        );
    }

    for (o1, o2) in expected_matches {
        let opponent1 = players[*o1].clone();
        let opponent2 = players[*o2].clone();

        let (next_opponent, _) = bracket
            .next_opponent(opponent1.get_id())
            .expect("next opponent")
            .expect("opponent should not be missing");
        let Opponent::Player(p) = next_opponent else {
            panic!("expected player for next opponent");
        };
        assert_eq!(
            p,
            opponent2.get_id(),
            "expected {opponent2} for {opponent1} but got {p}"
        );
        let (next_opponent, _) = bracket
            .next_opponent(opponent2.get_id())
            .expect("next opponent")
            .expect("opponent should not be missing");
        let Opponent::Player(p) = next_opponent else {
            panic!("expected player for next opponent");
        };
        assert_eq!(
            p,
            opponent1.get_id(),
            "expected {opponent1} for {opponent2} but got {p}"
        );
    }
}

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
fn bracket_5_man_with_frequent_upsets() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = Participants::default();
    for i in 1..=5 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding = seeding.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_seeding()).unwrap(),
        AutomaticMatchValidationMode::Strict,
    );

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[4].get_id(), (2, 0), p[5].get_id())
        .expect("winner 4vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[5], &[(1, 4), (2, 3)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (0, 2), p[4].get_id())
        .expect("winner 1vs4");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[4], &[(2, 3), (1, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (2, 0), p[3].get_id())
        .expect("winner 2vs3");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[3], &[(1, 5), (2, 4)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (0, 2), p[5].get_id())
        .expect("loser 1vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(2, 4), (3, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (2, 0), p[4].get_id())
        .expect("winner 2vs4");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert_next_matches(&bracket, &[2, 4], &[(3, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[3].get_id(), (0, 2), p[5].get_id())
        .expect("loser 3vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[2], &[(4, 5)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[4].get_id(), (2, 0), p[5].get_id())
        .expect("loser 4vs5");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(2, 4)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (0, 2), p[4].get_id())
        .expect("grand finals 2vs4");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 1);
    assert_next_matches(&bracket, &[], &[(2, 4)], &p);

    let (bracket, match_id, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (2, 0), p[4].get_id())
        .expect("reset 2vs4");
    let (bracket, new_matches) = bracket.validate_match_result(match_id);
    assert_eq!(new_matches.len(), 0);
    assert!(bracket.is_over());
    assert_no_next_match_after_tournament_is_over(&bracket);
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

    assert!(bracket.is_over());
    assert_no_next_match_after_tournament_is_over(&bracket);
}

#[test]
fn run_8_man_bracket_with_frequent_upsets2() {
    // every 2 matches, there is an upset
    let mut p = vec![Player::new("don't use".into())];
    let mut unpadded_p = vec![];
    let mut seeding = Participants::default();
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        unpadded_p.push(player.get_id());
        seeding = seeding.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_seeding()).unwrap(),
        AutomaticMatchValidationMode::Strict,
    );
    assert_eq!(bracket.get_matches().len(), 15);

    let (bracket, winner_1vs8, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (2, 0), p[8].get_id())
        .expect("bracket");
    let (bracket, _) = bracket.validate_match_result(winner_1vs8);
    let (bracket, winner_2vs7, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (0, 2), p[7].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(winner_2vs7);
    let (bracket, winner_3vs6, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[3].get_id(), (2, 0), p[6].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(winner_3vs6);
    let (bracket, winner_4vs5, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[4].get_id(), (0, 2), p[5].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(winner_4vs5);
    let (bracket, loser_4vs8, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[4].get_id(), (2, 0), p[8].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(loser_4vs8);
    let (bracket, loser_2vs6, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (0, 2), p[6].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(loser_2vs6);
    let (bracket, winner_1vs5, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (2, 0), p[5].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(winner_1vs5);
    let (bracket, winner_3vs7, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[3].get_id(), (0, 2), p[7].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(winner_3vs7);
    let (bracket, loser_3vs6, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[3].get_id(), (2, 0), p[6].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(loser_3vs6);
    let (bracket, loser_4vs5, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[4].get_id(), (0, 2), p[5].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(loser_4vs5);
    let (bracket, loser_3vs5, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[3].get_id(), (2, 0), p[5].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(loser_3vs5);
    let (bracket, winner_1vs7, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (0, 2), p[7].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(winner_1vs7);
    let (bracket, loser_1vs3, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (2, 0), p[3].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(loser_1vs3);
    let (bracket, grand_finals, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), (0, 2), p[7].get_id())
        .expect("s");
    let (bracket, _) = bracket.validate_match_result(grand_finals);
    assert!(bracket.is_over());
    assert_no_next_match_after_tournament_is_over(&bracket);
}
