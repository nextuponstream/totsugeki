// test /bracket/report

pub mod common;

use chrono::prelude::*;
use common::{
    bracket::{
        create_bracket, players_join_new_bracket_and_bracket_starts,
        tournament_organiser_starts_bracket,
    },
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
use test_log::test;
use totsugeki::{
    bracket::Format,
    matches::{MatchResultPOST, ReportedResult},
    player::Id as PlayerId,
    seeding::Method,
};
use totsugeki_api::Service;
use tracing::{debug, trace};

#[test(tokio::test)]
async fn participants_cannot_report_result_until_bracket_starts() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

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
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
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

        // Reporting a match result before bracket starts returns an error
        let body_result = MatchResultPOST {
            player_internal_id: "2".to_string(),
            channel_internal_id: channel_internal_id.to_string(),
            service_type_id: service.to_string(),
            result: "2-0".to_string(),
        };
        let res = test_api
            .cli
            .post("/bracket/report")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body_result)
            .send()
            .await;
        res.assert_status(StatusCode::UNAUTHORIZED);
        res.assert_text(format!(
            "Bracket \"{}\" does not accept match result reports",
            bracket.bracket_id
        ))
        .await;

        // After a tournament organiser start the bracket
        tournament_organiser_starts_bracket(&test_api, channel_internal_id, service).await;

        // Then you can start reporting results
        trace!("Organiser has started bracket. Reporting result should now work");
        let res = test_api
            .cli
            .post("/bracket/report")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body_result)
            .send()
            .await;
        res.assert_status(StatusCode::OK);
    }
}

#[tokio::test]
async fn match_results_cannot_be_reported_once_bracket_has_ended() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        let organiser_id = "1";
        let channel_internal_id = "1";
        let service = Service::Discord;
        let (bracket, _) = players_join_new_bracket_and_bracket_starts(
            &test_api,
            organiser_id,
            channel_internal_id,
            service,
            Format::SingleElimination,
            Method::Strict,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            3,
            false,
        )
        .await;

        both_player_report_match_result(
            &test_api,
            "2",
            "3",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match_for_predicted_seeds(&test_api, 2, 3, &bracket.matches).await;

        both_player_report_match_result(
            &test_api,
            "1",
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        validate_match_for_predicted_seeds(&test_api, 1, 2, &bracket.matches).await;

        // Reporting a match is unauthorized once bracket has ended
        let body = MatchResultPOST {
            player_internal_id: "1".to_string(),
            channel_internal_id: channel_internal_id.to_string(),
            service_type_id: service.to_string(),
            result: ReportedResult((2, 0)).to_string(),
        };
        let res = test_api
            .cli
            .post("/bracket/report")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body)
            .send()
            .await;
        res.assert_status(StatusCode::UNAUTHORIZED);
        res.assert_text(format!(
            "Bracket \"{}\" does not accept match result reports",
            bracket.bracket_id
        ))
        .await;
    }
}

#[test(tokio::test)]
async fn reporting_result_for_first_round_3_man() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        let organiser_id = "1";
        let channel_internal_id = "1";
        let service = Service::Discord;
        let (bracket, _) = players_join_new_bracket_and_bracket_starts(
            &test_api,
            organiser_id,
            channel_internal_id,
            service,
            Format::SingleElimination,
            Method::Strict,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            3,
            false,
        )
        .await;
        debug!("Bracket matches (start): {:?}", bracket.matches);

        let body = MatchResultPOST {
            player_internal_id: "1".to_string(),
            channel_internal_id: channel_internal_id.to_string(),
            service_type_id: service.to_string(),
            result: ReportedResult((2, 0)).to_string(),
        };
        let res = test_api
            .cli
            .post("/bracket/report")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body)
            .send()
            .await;
        let player1 = bracket.players[0].get_id();
        trace!("Top seed reporting a match should have no effect since he has not opponent yet");
        res.assert_status(StatusCode::FORBIDDEN);
        res.assert_text(format!("Bracket cannot be updated: Cannot report result in a match where opponent is missing. Current players: {player1} VS ?")).await;

        both_player_report_match_result(
            &test_api,
            "2",
            "3",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;

        trace!("Advancing bracket");
        validate_match_for_predicted_seeds(&test_api, 2, 3, &bracket.matches).await;

        trace!("Player 1 and 2 should be playing");
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

        assert_player_is_eliminated(&test_api, 3, channel_internal_id, service).await;
    }
}

#[test(tokio::test)]
async fn running_5_man_single_elimination_api() {
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
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        )
        .await;
        tournament_organiser_starts_bracket(&test_api, channel_internal_id, service).await;

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
async fn running_8_man_single_elimination_bracket() {
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
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        )
        .await;
        tournament_organiser_starts_bracket(&test_api, channel_internal_id, service).await;

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

#[test(tokio::test)]
async fn running_9_man_single_elimination_bracket() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        let organiser_internal_id = "1";
        let channel_internal_id = "1";
        let format = Format::SingleElimination;
        let seeding_method = Method::Strict;
        let service = Service::Discord;
        let (bracket, _organiser_name) = players_join_new_bracket_and_bracket_starts(
            &test_api,
            organiser_internal_id,
            channel_internal_id,
            service,
            format,
            seeding_method,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            9,
            false,
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
