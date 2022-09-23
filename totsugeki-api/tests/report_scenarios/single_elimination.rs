//! Reporting for single elimination bracket

use crate::common::{
    bracket::players_join_new_bracket_and_bracket_starts,
    db_types_to_test,
    next_match::{
        assert_next_matches, assert_player_has_no_next_match,
        assert_player_is_eliminated_from_bracket,
    },
    report::both_player_report_match_result,
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

        // player 5 wins against 4. Both players report their results
        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            channel_internal_id,
            service,
            (2, 0),
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
        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "1",
            "5",
            channel_internal_id,
            service,
            (2, 1),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "3",
            "2",
            channel_internal_id,
            service,
            (2, 0),
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
        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "3",
            "1",
            channel_internal_id,
            service,
            (2, 0),
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
async fn run_5_man_bracket_with_automatic_match_validation() {
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
            (2, 0),
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
            (2, 1),
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
            (2, 0),
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
            (2, 0),
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
async fn run_8_man_bracket() {
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "1",
            "8",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "2",
            "7",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            channel_internal_id,
            service,
            (2, 1),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "5",
            "1",
            channel_internal_id,
            service,
            (2, 1),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "6",
            "3",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "6",
            "2",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "5",
            "6",
            channel_internal_id,
            service,
            (2, 0),
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
async fn run_8_man_bracket_with_automatic_match_validation() {
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
            (2, 0),
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
            (2, 0),
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
            (2, 1),
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
            (2, 1),
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
            (2, 0),
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
            (2, 0),
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
            (2, 0),
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
async fn run_9_man_bracket() {
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "9",
            "8",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "3",
            "6",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "7",
            "2",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "3",
            "7",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "9",
            "1",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "9",
            "5",
            channel_internal_id,
            service,
            (2, 0),
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

        let (match_id, _) = both_player_report_match_result(
            &test_api,
            "3",
            "9",
            channel_internal_id,
            service,
            (2, 0),
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
async fn run_9_man_bracket_with_automatic_match_validation() {
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
            (2, 0),
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
            (2, 0),
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
            (2, 0),
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
            (2, 0),
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
            (2, 0),
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
            (2, 0),
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
            (2, 0),
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
            (2, 0),
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
