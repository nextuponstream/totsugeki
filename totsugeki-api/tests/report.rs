// test /bracket/report

pub mod common;

use common::{
    bracket::create_bracket,
    db_types_to_test,
    join::n_players_join_bracket,
    next_match::{assert_next_matches, assert_player_is_eliminated},
    report::player_reports_match_result,
    test_api,
    validate::validate_match_for_predicted_seeds,
};
use poem::http::StatusCode;
use totsugeki::{
    bracket::Format, matches::MatchResultPOST, player::Id as PlayerId, seeding::Method,
};
use totsugeki_api::Service;

#[tokio::test]
async fn reporting_result_for_first_round_3_man() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        // A bracket exists
        let organiser_id = "1";
        let channel_internal_id = "1";
        let service_type_id = Service::Discord;
        let (bracket_post_resp, _bracket_name, _) = create_bracket(
            &test_api,
            organiser_id,
            channel_internal_id,
            service_type_id,
            Format::SingleElimination,
            Method::Strict,
        )
        .await;

        let bracket = n_players_join_bracket(
            &test_api,
            channel_internal_id,
            service_type_id.to_string().as_str(),
            bracket_post_resp.get_bracket_id(),
        )
        .await;

        // Top seed reporting a match has no effect since he has not opponent yet
        let body = MatchResultPOST {
            player_internal_id: "1".to_string(),
            channel_internal_id: channel_internal_id.to_string(),
            service_type_id: service_type_id.to_string(),
            result: "2-0".to_string(),
        };
        let res = test_api
            .cli
            .post("/bracket/report")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body)
            .send()
            .await;
        res.assert_status(StatusCode::NOT_FOUND);

        player_reports_match_result(
            &test_api,
            "2",
            channel_internal_id,
            service_type_id.to_string().as_str(),
            "2-0",
        )
        .await;
        player_reports_match_result(
            &test_api,
            "3",
            channel_internal_id,
            service_type_id.to_string().as_str(),
            "0-2",
        )
        .await;

        // When tournament organiser validates match from seed 2 and 3, the
        // bracket advances
        validate_match_for_predicted_seeds(&test_api, 2, 3, bracket.matches).await;

        // parse bracket for players list
        assert_next_matches(
            &[],
            &[(1, 2)],
            &bracket
                .players
                .iter()
                .map(|p| p.get_id())
                .collect::<Vec<PlayerId>>(),
            &test_api,
            "1".to_string(),
            "discord".to_string(),
        )
        .await;

        // use player list to assert next matches
        assert_player_is_eliminated(
            &test_api,
            3,
            channel_internal_id.to_string(),
            service_type_id.to_string(),
        )
        .await;
    }
}
