//! tests for double elimination bracket

#[cfg(test)]
mod tests {
    // use crate::{
    //     bracket::query_state::{
    //         assert_elimination, assert_next_matches, create_bracket_with_n_players_and_start,
    //     },
    //     format::Format,
    //     seeding::Method,
    // };

    // const FORMAT: Format = Format::DoubleElimination;

    // #[test]
    // fn bracket_5_man() {
    //     let (bracket, players) =
    //         create_bracket_with_n_players_and_start(5, FORMAT, Method::Strict, false);

    //     let (bracket, match_id, _) = bracket
    //         .tournament_organiser_reports_result(players[5].get_id(), (2, 0), players[4].get_id())
    //         .expect("winner 4vs5");
    //     let bracket = bracket.validate_match_result(match_id).expect("validation");
    //     assert_next_matches(&bracket, &[], &[(1, 5), (2, 3)], &players);

    //     let (bracket, match_id, _) = bracket
    //         .tournament_organiser_reports_result(players[1].get_id(), (2, 1), players[5].get_id())
    //         .expect("winner 1vs5");
    //     let bracket = bracket.validate_match_result(match_id).expect("validation");
    //     assert_next_matches(&bracket, &[1], &[(2, 3)], &players);

    //     let (bracket, match_id, _) = bracket
    //         .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[2].get_id())
    //         .expect("winner 2vs3");
    //     let bracket = bracket.validate_match_result(match_id).expect("validation");
    //     assert_next_matches(&bracket, &[], &[(1, 3)], &players);

    //     let (bracket, match_id, _) = bracket
    //         .tournament_organiser_reports_result(players[3].get_id(), (2, 0), players[1].get_id())
    //         .expect("winner 1vs3");
    //     let bracket = bracket.validate_match_result(match_id).expect("validation");

    //     assert_elimination(&bracket, &players, 3);
    // }
}
