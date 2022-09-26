//! tests to verify state of single elimination bracket

#[cfg(test)]
mod tests {
    use crate::{
        bracket::query_state::{
            assert_elimination, assert_next_matches, create_bracket_with_n_players_and_start,
        },
        format::Format,
        seeding::Method,
    };

    const FORMAT: Format = Format::SingleElimination;

    #[test]
    fn bracket_5_man() {
        let (bracket, players) =
            create_bracket_with_n_players_and_start(5, FORMAT, Method::Strict, false);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[4].get_id())
            .expect("winner 4vs5");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(1, 5), (2, 3)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[1].get_id(), (2, 1), players[5].get_id())
            .expect("winner 1vs5");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1], &[(2, 3)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[2].get_id())
            .expect("winner 2vs3");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(1, 3)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[1].get_id())
            .expect("winner 1vs3");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&bracket, &players, 3);
    }

    #[test]
    fn bracket_5_man_automated() {
        let (bracket, players) =
            create_bracket_with_n_players_and_start(5, FORMAT, Method::Strict, true);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[4].get_id())
            .expect("winner 4vs5");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(1, 5), (2, 3)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[1].get_id(), (2, 1), players[5].get_id())
            .expect("winner 1vs5");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1], &[(2, 3)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[2].get_id())
            .expect("winner 2vs3");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(1, 3)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[1].get_id())
            .expect("winner 1vs3");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&bracket, &players, 3);
    }

    #[test]
    fn bracket_8_man() {
        let (bracket, players) =
            create_bracket_with_n_players_and_start(8, FORMAT, Method::Strict, false);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[1].get_id(), (2, 0), players[8].get_id())
            .expect("winner 1vs8");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1], &[(2, 7), (3, 6), (4, 5)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[2].get_id(), (2, 0), players[7].get_id())
            .expect("winner 2vs7");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1, 2], &[(3, 6), (4, 5)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[4].get_id())
            .expect("winner 4vs5");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[2], &[(3, 6), (1, 5)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[1].get_id())
            .expect("winner 1vs5");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[2, 5], &[(3, 6)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[6].get_id(), (2, 0), players[3].get_id())
            .expect("winner 3vs6");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(2, 6)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[6].get_id(), (2, 0), players[2].get_id())
            .expect("winner 2vs6");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(5, 6)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[6].get_id())
            .expect("winner 5vs6");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&bracket, &players, 5);
    }

    #[test]
    fn bracket_8_man_automated() {
        let (bracket, players) =
            create_bracket_with_n_players_and_start(8, FORMAT, Method::Strict, true);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[1].get_id(), (2, 0), players[8].get_id())
            .expect("winner 1vs8");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1], &[(2, 7), (3, 6), (4, 5)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[2].get_id(), (2, 0), players[7].get_id())
            .expect("winner 2vs7");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1, 2], &[(3, 6), (4, 5)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[4].get_id())
            .expect("winner 4vs5");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[2], &[(3, 6), (1, 5)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[1].get_id())
            .expect("winner 1vs5");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[2, 5], &[(3, 6)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[6].get_id(), (2, 0), players[3].get_id())
            .expect("winner 3vs6");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(2, 6)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[6].get_id(), (2, 0), players[2].get_id())
            .expect("winner 2vs6");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(5, 6)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[6].get_id())
            .expect("winner 5vs6");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&bracket, &players, 5);
    }

    #[test]
    fn bracket_9_man() {
        let (bracket, players) =
            create_bracket_with_n_players_and_start(9, FORMAT, Method::Strict, false);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[4].get_id())
            .expect("winner 4vs5");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1, 5], &[(8, 9), (3, 6), (2, 7)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[9].get_id(), (2, 0), players[8].get_id())
            .expect("winner 8vs9");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(1, 9), (3, 6), (2, 7)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[6].get_id())
            .expect("winner 3vs6");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[3, 5], &[(1, 9), (2, 7)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[7].get_id(), (2, 0), players[2].get_id())
            .expect("winner 3vs6");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(1, 9), (3, 7)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[7].get_id())
            .expect("winner 3vs7");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[3, 5], &[(1, 9)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[9].get_id(), (2, 0), players[1].get_id())
            .expect("winner 1vs9");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[3], &[(9, 5)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[9].get_id(), (2, 0), players[5].get_id())
            .expect("winner 5vs9");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(3, 9)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[9].get_id())
            .expect("winner 3vs9");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&bracket, &players, 3);
    }

    #[test]
    fn bracket_9_man_automated() {
        let (bracket, players) =
            create_bracket_with_n_players_and_start(9, FORMAT, Method::Strict, true);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[4].get_id())
            .expect("winner 4vs5");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1, 5], &[(8, 9), (3, 6), (2, 7)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[9].get_id(), (2, 0), players[8].get_id())
            .expect("winner 8vs9");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(1, 9), (3, 6), (2, 7)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[6].get_id())
            .expect("winner 3vs6");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[3, 5], &[(1, 9), (2, 7)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[7].get_id(), (2, 0), players[2].get_id())
            .expect("winner 3vs6");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(1, 9), (3, 7)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[7].get_id())
            .expect("winner 3vs7");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[3, 5], &[(1, 9)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[9].get_id(), (2, 0), players[1].get_id())
            .expect("winner 1vs9");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[3], &[(9, 5)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[9].get_id(), (2, 0), players[5].get_id())
            .expect("winner 5vs9");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(3, 9)], &players);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[9].get_id())
            .expect("winner 3vs9");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&bracket, &players, 3);
    }
}
