//! tests for double elimination bracket

#[cfg(test)]
mod tests {
    use crate::{
        bracket::query_state::{
            assert_elimination, assert_next_matches, create_bracket_with_n_players_and_start,
        },
        format::Format,
        seeding::Method,
    };

    const FORMAT: Format = Format::DoubleElimination;

    #[test]
    fn bracket_5_man_with_frequent_upsets() {
        let (bracket, players) =
            create_bracket_with_n_players_and_start(5, FORMAT, Method::Strict, false);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[4].get_id(), (2, 0), players[5].get_id())
            .expect("winner 4vs5");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(1, 4), (2, 3)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[1].get_id(), (0, 2), players[4].get_id())
            .expect("winner 1vs4");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[4], &[(2, 3), (1, 5)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[2].get_id(), (2, 0), players[3].get_id())
            .expect("winner 2vs3");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[3], &[(1, 5), (2, 4)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[1].get_id(), (0, 2), players[5].get_id())
            .expect("loser 1vs5");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(2, 4), (3, 5)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[2].get_id(), (2, 0), players[4].get_id())
            .expect("winner 2vs4");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[2, 4], &[(3, 5)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[3].get_id(), (0, 2), players[5].get_id())
            .expect("loser 3vs5");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[2], &[(4, 5)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[4].get_id(), (2, 0), players[5].get_id())
            .expect("loser 4vs5");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(2, 4)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[2].get_id(), (0, 2), players[4].get_id())
            .expect("grand finals 2vs4");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(2, 4)], &players);

        let (bracket, match_id, _) = bracket
            .tournament_organiser_reports_result(players[2].get_id(), (2, 0), players[4].get_id())
            .expect("reset 2vs4");
        let (bracket, new_matches) = bracket.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&bracket, &players, 2);
    }
}
