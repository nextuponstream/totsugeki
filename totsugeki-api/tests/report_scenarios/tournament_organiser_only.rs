//! Only tournament organiser reports results

use crate::common::{
    bracket::players_join_new_bracket_and_bracket_starts,
    db_types_to_test,
    next_match::{
        assert_next_matches, assert_player_has_no_next_match,
        assert_player_is_eliminated_from_bracket,
    },
    report::tournament_organiser_reports_match_result,
    test_api,
    validate::validate_match,
};
use chrono::prelude::*;
use test_log::test;
use totsugeki::{format::Format, player::Id as PlayerId, seeding::Method};
use totsugeki_api::Service;

#[test(tokio::test)]
async fn run_5_man_bracket() {
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

        let (match_id, _) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(4).expect("p5").to_string().as_str(),
            (2, 0),
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
        let (match_id, _) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(0).expect("p1").to_string().as_str(),
            (2, 0),
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

        let (match_id, _) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(2).expect("p3").to_string().as_str(),
            (2, 0),
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
        let (match_id, _) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            players.get(2).expect("p3").to_string().as_str(),
            (2, 0),
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
                    &bracket.players[s - 1],
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                    &bracket.players[s - 1],
                )
                .await;
            }
        }
    }
}

#[test(tokio::test)]
async fn run_5_man_bracket_automatically() {
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
            (2, 0),
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
            (2, 0),
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
            (2, 0),
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
            (2, 0),
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
                    &bracket.players[s - 1],
                )
                .await;
            } else {
                assert_player_is_eliminated_from_bracket(
                    &test_api,
                    s,
                    channel_internal_id,
                    service,
                    bracket.bracket_id,
                    &bracket.players[s - 1],
                )
                .await;
            }
        }
    }
}
