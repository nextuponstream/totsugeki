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
        assert_next_matches, assert_player_has_no_next_match,
        assert_player_is_eliminated_from_bracket,
    },
    report::{
        both_player_report_match_result, player_reports_match_result,
        tournament_organiser_reports_match_result,
    },
    test_api,
    validate::{validate_match, validate_match_for_predicted_seeds},
};
use poem::http::StatusCode;
use test_log::test;
use totsugeki::{
    bracket::http_responses::GET,
    format::Format,
    matches::{PlayerMatchResultPOST, ReportedResult},
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
        let player1 = bracket.players[0].get_id();
        trace!("Top seed reporting a match should have no effect since he has not opponent yet");
        res.assert_status(StatusCode::FORBIDDEN);
        res.assert_text(format!("Action is forbidden:\n\tCannot report result in a match where opponent is missing. Current players: {player1} VS ?")).await;

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
        let player1 = bracket.players[0].get_id();
        trace!("Top seed reporting a match should have no effect since he has not opponent yet");
        res.assert_status(StatusCode::FORBIDDEN);
        res.assert_text(format!("Action is forbidden:\n\tCannot report result in a match where opponent is missing. Current players: {player1} VS ?")).await;

        let response = player_reports_match_result(
            &test_api,
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_eq!(response.message, "Result reported".to_string());
        let response = player_reports_match_result(
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

#[test(tokio::test)]
async fn running_5_man_single_elimination_api() {
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
            5,
            false,
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
                assert_player_has_no_next_match(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            }
        }
    }
}

#[test(tokio::test)]
async fn tournament_organiser_runs_5_man_tournament() {
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
            5,
            false,
        )
        .await;
        let players = bracket
            .players
            .iter()
            .map(|p| p.id)
            .collect::<Vec<PlayerId>>();

        let match_id = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(4).expect("p5").to_string().as_str(),
            ReportedResult((2, 0)),
            players.get(3).expect("p4").to_string().as_str(),
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
        let match_id = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(0).expect("p1").to_string().as_str(),
            ReportedResult((2, 0)),
            players.get(4).expect("p5").to_string().as_str(),
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

        let match_id = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(2).expect("p3").to_string().as_str(),
            ReportedResult((2, 0)),
            players.get(1).expect("p2").to_string().as_str(),
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
        let match_id = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(2).expect("p3").to_string().as_str(),
            ReportedResult((2, 0)),
            players.get(0).expect("p1").to_string().as_str(),
        )
        .await;
        validate_match(&test_api, match_id).await;

        // noone has matches
        for s in 1..=5 {
            if s == 3 {
                assert_player_has_no_next_match(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            }
        }
    }
}

#[test(tokio::test)]
async fn tournament_organiser_runs_5_man_tournament_automatically() {
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
            5,
            true,
        )
        .await;
        let players = bracket
            .players
            .iter()
            .map(|p| p.id)
            .collect::<Vec<PlayerId>>();

        let _match_id = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(4).expect("p5").to_string().as_str(),
            ReportedResult((2, 0)),
            players.get(3).expect("p4").to_string().as_str(),
        )
        .await;

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
        let _match_id = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(0).expect("p1").to_string().as_str(),
            ReportedResult((2, 0)),
            players.get(4).expect("p5").to_string().as_str(),
        )
        .await;
        assert_next_matches(
            &[1],
            &[(2, 3)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _match_id = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(2).expect("p3").to_string().as_str(),
            ReportedResult((2, 0)),
            players.get(1).expect("p2").to_string().as_str(),
        )
        .await;

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
        let _match_id = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(2).expect("p3").to_string().as_str(),
            ReportedResult((2, 0)),
            players.get(0).expect("p1").to_string().as_str(),
        )
        .await;

        // noone has matches
        for s in 1..=5 {
            if s == 3 {
                assert_player_has_no_next_match(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            }
        }
    }
}

#[test(tokio::test)]
async fn running_5_man_single_elimination_api_with_automatic_match_validation() {
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
            5,
            true,
        )
        .await;
        let players = bracket
            .players
            .iter()
            .map(|p| p.id)
            .collect::<Vec<PlayerId>>();

        // player 5 wins against 4. Both players report their results
        let _ = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;

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
        let _ = both_player_report_match_result(
            &test_api,
            "1",
            "5",
            channel_internal_id,
            service,
            ReportedResult((2, 1)),
        )
        .await;
        assert_next_matches(
            &[1],
            &[(2, 3)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "3",
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[],
            &[(1, 3)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "3",
            "1",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;

        // noone has matches
        for s in 1..=5 {
            if s == 3 {
                assert_player_has_no_next_match(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            }
        }
    }
}

#[test(tokio::test)]
async fn running_8_man_single_elimination_bracket() {
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
            8,
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
                assert_player_has_no_next_match(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            }
        }
    }
}

#[test(tokio::test)]
async fn running_8_man_single_elimination_bracket_with_automatic_match_validation() {
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
            8,
            true,
        )
        .await;

        let players = bracket
            .players
            .iter()
            .map(|p| p.id)
            .collect::<Vec<PlayerId>>();

        let _ = both_player_report_match_result(
            &test_api,
            "1",
            "8",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[1],
            &[(2, 7), (3, 6), (4, 5)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "2",
            "7",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[1, 2],
            &[(3, 6), (4, 5)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            channel_internal_id,
            service,
            ReportedResult((2, 1)),
        )
        .await;
        assert_next_matches(
            &[2],
            &[(1, 5), (3, 6)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "5",
            "1",
            channel_internal_id,
            service,
            ReportedResult((2, 1)),
        )
        .await;
        assert_next_matches(
            &[2, 5],
            &[(3, 6)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "6",
            "3",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[5],
            &[(2, 6)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "6",
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[],
            &[(5, 6)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "5",
            "6",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;

        // noone has matches
        for s in 1..=8 {
            if s == 5 {
                assert_player_has_no_next_match(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
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
                assert_player_has_no_next_match(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            }
        }
    }
}

#[test(tokio::test)]
async fn running_9_man_single_elimination_bracket_with_automatic_match_validation() {
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
            true,
        )
        .await;

        let players = bracket
            .players
            .iter()
            .map(|p| p.id)
            .collect::<Vec<PlayerId>>();

        let _ = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[1, 5],
            &[(8, 9), (3, 6), (2, 7)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "9",
            "8",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[5],
            &[(1, 9), (3, 6), (2, 7)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "3",
            "6",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[3, 5],
            &[(1, 9), (2, 7)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "7",
            "2",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[5],
            &[(1, 9), (3, 7)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "3",
            "7",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[3, 5],
            &[(1, 9)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "9",
            "1",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[3],
            &[(9, 5)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "9",
            "5",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;
        assert_next_matches(
            &[],
            &[(3, 9)],
            &players,
            &test_api,
            channel_internal_id.to_string(),
            service,
        )
        .await;

        let _ = both_player_report_match_result(
            &test_api,
            "3",
            "9",
            channel_internal_id,
            service,
            ReportedResult((2, 0)),
        )
        .await;

        // noone has matches
        for s in 1..=9 {
            if s == 3 {
                assert_player_has_no_next_match(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                )
                .await;
            }
        }
    }
}
