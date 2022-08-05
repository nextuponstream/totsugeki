// test /bracket/report

pub mod common;

use common::{
    bracket::create_bracket,
    db_types_to_test,
    join::n_players_join_bracket,
    next_match::{
        assert_next_matches, assert_player_has_no_next_match, assert_player_is_eliminated,
    },
    report::both_player_report_match_result,
    test_api,
    validate::{validate_match, validate_match_for_predicted_seeds},
};
use poem::http::StatusCode;
use totsugeki::{
    bracket::Format,
    matches::{MatchResultPOST, ReportedResult},
    player::Id as PlayerId,
    seeding::Method,
};
use totsugeki_api::Service;

#[tokio::test]
async fn reporting_result_for_first_round_3_man() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        // A bracket exists
        let organiser_id = "1";
        let channel_internal_id = "1";
        let service = Service::Discord;
        let (bracket_post_resp, _bracket_name, _) = create_bracket(
            &test_api,
            organiser_id,
            channel_internal_id,
            service,
            Format::SingleElimination,
            Method::Strict,
        )
        .await;

        let bracket = n_players_join_bracket(
            &test_api,
            3,
            channel_internal_id,
            service,
            bracket_post_resp.get_bracket_id(),
        )
        .await;

        // Top seed reporting a match has no effect since he has not opponent yet
        let body = MatchResultPOST {
            player_internal_id: "1".to_string(),
            channel_internal_id: channel_internal_id.to_string(),
            service_type_id: service.to_string(),
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

        both_player_report_match_result(
            &test_api,
            "2",
            "3",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
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
            channel_internal_id.to_string(),
            service,
        )
        .await;

        // use player list to assert next matches
        assert_player_is_eliminated(&test_api, 3, channel_internal_id, service).await;
    }
}

#[tokio::test]
async fn running_5_man_single_elimination_tournament_tournament() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        let organiser_internal_id = "1";
        let channel_internal_id = "1";
        let format = Format::SingleElimination;
        let seeding_method = Method::Strict;
        let service = Service::Discord;
        let (bracket, _bracket_name, _organiser_name) = create_bracket(
            &test_api,
            organiser_internal_id,
            channel_internal_id,
            service,
            format,
            seeding_method,
        )
        .await;

        let bracket = n_players_join_bracket(
            &test_api,
            5,
            channel_internal_id,
            service,
            bracket.get_bracket_id(),
        )
        .await;
        let players = bracket
            .players
            .iter()
            .map(|p| p.id)
            .collect::<Vec<PlayerId>>();

        // player 5 wins against 4. Both players report their results
        let match_id = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;

        assert_next_matches(
            &[],
            &[(1, 5), (2, 3)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        // player 1 wins against 5, 3 wins against 2
        let match_id = both_player_report_match_result(
            &test_api,
            "1",
            "5",
            channel_internal_id,
            service,
            ReportedResult((2, 1)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[1],
            &[(2, 3)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "3",
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;

        assert_next_matches(
            &[],
            &[(1, 3)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        // player 3 wins against seed 1
        let match_id = both_player_report_match_result(
            &test_api,
            "3",
            "1",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;

        // noone has matches
        for s in 1..=5 {
            if s == 3 {
                assert_player_has_no_next_match(&test_api, s, channel_internal_id, service).await;
            } else {
                assert_player_is_eliminated(&test_api, s, channel_internal_id, service).await;
            }
        }
    }
}

#[tokio::test]
async fn running_8_man_single_elimination_tournament_tournament() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        let organiser_internal_id = "1";
        let channel_internal_id = "1";
        let format = Format::SingleElimination;
        let seeding_method = Method::Strict;
        let service = Service::Discord;
        let (bracket, _bracket_name, _organiser_name) = create_bracket(
            &test_api,
            organiser_internal_id,
            channel_internal_id,
            service,
            format,
            seeding_method,
        )
        .await;

        let bracket = n_players_join_bracket(
            &test_api,
            8,
            channel_internal_id,
            service,
            bracket.get_bracket_id(),
        )
        .await;
        let players = bracket
            .players
            .iter()
            .map(|p| p.id)
            .collect::<Vec<PlayerId>>();

        let match_id = both_player_report_match_result(
            &test_api,
            "1",
            "8",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[1],
            &[(2, 7), (3, 6), (4, 5)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "2",
            "7",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[1, 2],
            &[(3, 6), (4, 5)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            channel_internal_id,
            service,
            ReportedResult((2, 1)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[2],
            &[(1, 5), (3, 6)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "5",
            "1",
            channel_internal_id,
            service,
            ReportedResult((2, 1)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[2, 5],
            &[(3, 6)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "6",
            "3",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[5],
            &[(2, 6)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "6",
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[],
            &[(5, 6)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "5",
            "6",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;

        // noone has matches
        for s in 1..=8 {
            if s == 5 {
                assert_player_has_no_next_match(&test_api, s, channel_internal_id, service).await;
            } else {
                assert_player_is_eliminated(&test_api, s, channel_internal_id, service).await;
            }
        }
    }
}

#[tokio::test]
async fn running_9_man_single_elimination_tournament_tournament() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        let organiser_internal_id = "1";
        let channel_internal_id = "1";
        let format = Format::SingleElimination;
        let seeding_method = Method::Strict;
        let service = Service::Discord;
        let (bracket, _bracket_name, _organiser_name) = create_bracket(
            &test_api,
            organiser_internal_id,
            channel_internal_id,
            service,
            format,
            seeding_method,
        )
        .await;

        let bracket = n_players_join_bracket(
            &test_api,
            9,
            channel_internal_id,
            service,
            bracket.get_bracket_id(),
        )
        .await;
        let players = bracket
            .players
            .iter()
            .map(|p| p.id)
            .collect::<Vec<PlayerId>>();

        let match_id = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[1, 5],
            &[(8, 9), (3, 6), (2, 7)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "9",
            "8",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[5],
            &[(1, 9), (3, 6), (2, 7)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "3",
            "6",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[3, 5],
            &[(1, 9), (2, 7)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "7",
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[5],
            &[(1, 9), (3, 7)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "3",
            "7",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[3, 5],
            &[(1, 9)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "9",
            "1",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[3],
            &[(9, 5)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "9",
            "5",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;
        assert_next_matches(
            &[],
            &[(3, 9)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let match_id = both_player_report_match_result(
            &test_api,
            "3",
            "9",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match(&test_api, match_id).await;

        // noone has matches
        for s in 1..=9 {
            if s == 3 {
                assert_player_has_no_next_match(&test_api, s, channel_internal_id, service).await;
            } else {
                assert_player_is_eliminated(&test_api, s, channel_internal_id, service).await;
            }
        }
    }
}
