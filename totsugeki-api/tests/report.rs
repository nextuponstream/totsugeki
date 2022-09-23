//! test /bracket/report

pub mod common;
mod report_scenarios;

use chrono::prelude::*;
use common::{
    bracket::{
        create_bracket, players_join_new_bracket_and_bracket_starts,
        tournament_organiser_starts_bracket,
    },
    db_types_to_test,
    join::n_players_join_bracket,
    next_match::{assert_next_matches, assert_player_is_eliminated_from_bracket},
    report::{both_player_report_match_result, player_reports_match_result},
    test_api,
    validate::validate_match_for_predicted_seeds,
};
use poem::http::StatusCode;
use test_log::test;
use totsugeki::{
    bracket::http_responses::GET,
    format::Format,
    matches::{PlayerMatchResultPOST, ReportedResult},
    opponent::Opponent,
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
            bracket_post_resp.bracket_id,
        )
        .await;

        // Reporting a match result before bracket starts returns an error
        let body_result = PlayerMatchResultPOST {
            internal_player_id: "2".to_string(),
            internal_channel_id: channel_internal_id.to_string(),
            service: service.to_string(),
            result: "2-0".to_string(),
        };
        let res = test_api
            .cli
            .post("/bracket/report/player")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body_result)
            .send()
            .await;
        res.assert_status(StatusCode::FORBIDDEN);
        res.assert_text(format!(
            "Action is forbidden:\n\tBracket \"{}\" does not accept reported results at the moment",
            bracket.bracket_id
        ))
        .await;

        // After a tournament organiser start the bracket
        tournament_organiser_starts_bracket(&test_api, channel_internal_id, service).await;

        // Then you can start reporting results
        trace!("Organiser has started bracket. Reporting result should now work");
        let res = test_api
            .cli
            .post("/bracket/report/player")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body_result)
            .send()
            .await;
        res.assert_status(StatusCode::OK);
    }
}

#[test(tokio::test)]
async fn match_results_cannot_be_reported_once_bracket_has_ended() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        let organiser_id = "1";
        let channel_internal_id = "1";
        let service = Service::Discord;
        let (bracket, _): (GET, String) = players_join_new_bracket_and_bracket_starts(
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

        both_player_report_match_result(&test_api, "2", "3", channel_internal_id, service, (2, 0))
            .await;
        validate_match_for_predicted_seeds(&test_api, 2, 3, &bracket.matches).await;

        both_player_report_match_result(&test_api, "1", "2", channel_internal_id, service, (2, 0))
            .await;
        validate_match_for_predicted_seeds(&test_api, 1, 2, &bracket.matches).await;

        // Reporting a match is forbidden once bracket has ended
        let body = PlayerMatchResultPOST {
            internal_player_id: "1".to_string(),
            internal_channel_id: channel_internal_id.to_string(),
            service: service.to_string(),
            result: ReportedResult((2, 0)).to_string(),
        };
        let res = test_api
            .cli
            .post("/bracket/report/player")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body)
            .send()
            .await;
        res.assert_status(StatusCode::FORBIDDEN);
        res.assert_text(format!(
            "Action is forbidden:\n\tBracket \"{}\" does not accept reported results at the moment",
            bracket.bracket_id
        ))
        .await;
    }
}

#[test(tokio::test)]
async fn players_disagreeing_on_a_result_allows_correction_from_both_side() {
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
            true,
        )
        .await;

        player_reports_match_result(
            &test_api,
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        player_reports_match_result(
            &test_api,
            "3",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        // Then they agree
        player_reports_match_result(
            &test_api,
            "2",
            channel_internal_id,
            service,
            ReportedResult((1, 2)),
        )
        .await;
        player_reports_match_result(
            &test_api,
            "3",
            channel_internal_id,
            service,
            ReportedResult((2, 1)),
        )
        .await;

        assert_next_matches(
            &[],
            &[(1, 3)],
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

        assert_player_is_eliminated_from_bracket(
            &test_api,
            2,
            channel_internal_id,
            service,
            bracket.bracket_id,
        )
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
        let (bracket, _): (GET, String) = players_join_new_bracket_and_bracket_starts(
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

        let body = PlayerMatchResultPOST {
            internal_player_id: "1".to_string(),
            internal_channel_id: channel_internal_id.to_string(),
            service: service.to_string(),
            result: ReportedResult((2, 0)).to_string(),
        };
        let res = test_api
            .cli
            .post("/bracket/report/player")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body)
            .send()
            .await;
        let player1 = Opponent::Player(bracket.players[0].clone());
        trace!("Top seed reporting a match should have no effect since he has not opponent yet");
        res.assert_status(StatusCode::FORBIDDEN);
        res.assert_text(format!("Action is forbidden:\n\tCannot report result in a match where opponent is missing. Current players: {player1} VS ?")).await;

        both_player_report_match_result(&test_api, "2", "3", channel_internal_id, service, (2, 0))
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

        assert_player_is_eliminated_from_bracket(
            &test_api,
            3,
            channel_internal_id,
            service,
            bracket.bracket_id,
        )
        .await;
    }
}

#[test(tokio::test)]
async fn reporting_result_for_first_round_3_man_with_automatic_match_validation() {
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
            true,
        )
        .await;
        debug!("Bracket matches (start): {:?}", bracket.matches);

        let body = PlayerMatchResultPOST {
            internal_player_id: "1".to_string(),
            internal_channel_id: channel_internal_id.to_string(),
            service: service.to_string(),
            result: ReportedResult((2, 0)).to_string(),
        };
        let res = test_api
            .cli
            .post("/bracket/report/player")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body)
            .send()
            .await;
        let player1 = Opponent::Player(bracket.players[0].clone());
        trace!("Top seed reporting a match should have no effect since he has not opponent yet");
        res.assert_status(StatusCode::FORBIDDEN);
        res.assert_text(format!("Action is forbidden:\n\tCannot report result in a match where opponent is missing. Current players: {player1} VS ?")).await;

        let (response, _) = player_reports_match_result(
            &test_api,
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_eq!(response.message, "Result reported".to_string());
        let (response, _) = player_reports_match_result(
            &test_api,
            "3",
            channel_internal_id,
            service,
            ReportedResult((0, 2)),
        )
        .await;
        assert_eq!(
            response.message,
            "Result reported and match validated".to_string()
        );

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

        assert_player_is_eliminated_from_bracket(
            &test_api,
            3,
            channel_internal_id,
            service,
            bracket.bracket_id,
        )
        .await;
    }
}
