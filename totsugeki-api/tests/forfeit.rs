// test /bracket/forfeit

pub mod common;

use chrono::prelude::*;
use common::{
    bracket::players_join_new_bracket_and_bracket_starts, db_types_to_test,
    forfeit::request as forfeit, report::both_player_report_match_result, test_api,
    validate::validate_match,
};
use totsugeki::{format::Format, matches::Match, opponent::Opponent, seeding::Method};
use totsugeki_api::Service;

#[tokio::test]
async fn player_forfeits_when_he_realises_he_cannot_make_it() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let internal_channel_id = "1";
        let service = Service::Discord;
        let (bracket, _) = players_join_new_bracket_and_bracket_starts(
            &test_api,
            "1",
            internal_channel_id,
            service,
            Format::default(),
            Method::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            5,
            false,
        )
        .await;

        let player_id = bracket.players.get(0).expect("player 1").get_id();
        let bracket = forfeit(
            &test_api,
            "1",
            internal_channel_id,
            service,
            bracket.bracket_id,
        )
        .await;

        let matches = bracket
            .matches
            .into_iter()
            .map(Match::try_from)
            .collect::<Result<Vec<_>, _>>()
            .expect("matches");
        assert!(matches.iter().any(|m| {
            if m.contains(player_id) {
                if let Opponent::Player(looser) = m.get_automatic_loser() {
                    return looser.get_id() == player_id;
                }
            }
            false
        }));

        let (match_id_seed_4, _) = both_player_report_match_result(
            &test_api,
            "4",
            "5",
            internal_channel_id,
            service,
            (2, 0),
        )
        .await;

        let (match_id_seed_2, _) = both_player_report_match_result(
            &test_api,
            "2",
            "3",
            internal_channel_id,
            service,
            (2, 0),
        )
        .await;

        validate_match(&test_api, match_id_seed_4).await;
        validate_match(&test_api, match_id_seed_2).await;

        let (gf, _) = both_player_report_match_result(
            &test_api,
            "2",
            "4",
            internal_channel_id,
            service,
            (2, 0),
        )
        .await;
        validate_match(&test_api, gf).await;

        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn player_has_to_go_mid_bracket() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let internal_channel_id = "1";
        let service = Service::Discord;
        let (bracket, _) = players_join_new_bracket_and_bracket_starts(
            &test_api,
            "1",
            internal_channel_id,
            service,
            Format::default(),
            Method::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            9,
            false,
        )
        .await;

        let (match_id_seed_8, _) = both_player_report_match_result(
            &test_api,
            "8",
            "9",
            internal_channel_id,
            service,
            (2, 0),
        )
        .await;
        let (match_id_seed_4, _) = both_player_report_match_result(
            &test_api,
            "5",
            "4",
            internal_channel_id,
            service,
            (2, 0),
        )
        .await;
        let (match_id_seed_3, _) = both_player_report_match_result(
            &test_api,
            "6",
            "3",
            internal_channel_id,
            service,
            (2, 0),
        )
        .await;
        let (match_id_seed_2, _) = both_player_report_match_result(
            &test_api,
            "2",
            "7",
            internal_channel_id,
            service,
            (2, 0),
        )
        .await;
        validate_match(&test_api, match_id_seed_8).await;
        validate_match(&test_api, match_id_seed_4).await;
        validate_match(&test_api, match_id_seed_3).await;
        validate_match(&test_api, match_id_seed_2).await;
        let (match_id_seed_1, _) = both_player_report_match_result(
            &test_api,
            "1",
            "8",
            internal_channel_id,
            service,
            (2, 0),
        )
        .await;
        validate_match(&test_api, match_id_seed_1).await;

        let bracket = forfeit(
            &test_api,
            "1",
            internal_channel_id,
            service,
            bracket.bracket_id,
        )
        .await;
        let _ = forfeit(
            &test_api,
            "6",
            internal_channel_id,
            service,
            bracket.bracket_id,
        )
        .await;
        let (gf, _) = both_player_report_match_result(
            &test_api,
            "5",
            "2",
            internal_channel_id,
            service,
            (2, 0),
        )
        .await;
        validate_match(&test_api, gf).await;

        test_api.clean_db().await;
    }
}
