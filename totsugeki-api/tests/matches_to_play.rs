// test /bracket/matches_to_play endpoint

pub mod common;
use crate::common::join::n_players_join_bracket;
use chrono::prelude::*;
use common::{
    bracket::{create_bracket, parse_matches},
    db_types_to_test,
    matches::{assert_if_matches_exist, assert_seeding},
    report::{both_player_report_match_result, tournament_organiser_reports_match_result},
    test_api,
};
use reqwest::StatusCode;
use test_log::test;
use totsugeki::{
    bracket::http_responses::CommandPOST,
    format::Format,
    player::{Id as PlayerId, Player},
    seeding::Method,
};
use totsugeki_api::Service;

#[test(tokio::test)]
async fn run_8_man_double_elimination() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let channel_internal_id = "1";
        let service = Service::Discord;
        let (bracket, _, _) = create_bracket(
            &test_api,
            "1",
            channel_internal_id,
            service,
            Format::DoubleElimination,
            Method::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            true,
        )
        .await;
        let bracket = n_players_join_bracket(
            &test_api,
            8,
            channel_internal_id,
            service,
            bracket.bracket_id,
        )
        .await;
        let mut player_ids = bracket
            .players
            .iter()
            .map(|m| m.get_id())
            .collect::<Vec<PlayerId>>();
        player_ids.reverse();
        player_ids.push(PlayerId::new_v4()); // padding for readability
        player_ids.reverse();
        let mut players = bracket.players.clone();
        players.reverse();
        players.push(Player::new("don't use".into())); // padding for readability
        players.reverse();

        let body_start = CommandPOST {
            channel_internal_id: channel_internal_id.to_string(),
            service_type_id: service.to_string(),
        };
        let res = test_api
            .cli
            .post("/bracket/start")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body_start)
            .send()
            .await;
        res.assert_status(StatusCode::OK);
        let resp = res.json().await;
        let resp = resp.value().object();
        let matches_to_play = parse_matches(&resp);
        assert_eq!(matches_to_play.len(), 4);

        assert_seeding(&[[1, 8], [2, 7], [3, 6], [4, 5]], &matches_to_play);
        assert_if_matches_exist(
            &[
                ([1, 8], [players[1].clone(), players[8].clone()]),
                ([2, 7], [players[2].clone(), players[7].clone()]),
                ([3, 6], [players[3].clone(), players[6].clone()]),
                ([4, 5], [players[4].clone(), players[5].clone()]),
            ],
            &matches_to_play,
        );

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[1].get_id().to_string(),
            (2, 0),
            &players[8].get_id().to_string(),
        )
        .await;
        assert_if_matches_exist(&[], &new_matches_to_play);

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[4].get_id().to_string(),
            (2, 0),
            &players[5].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 2);
        assert_if_matches_exist(
            &[
                ([1, 4], [players[1].clone(), players[4].clone()]),
                ([5, 8], [players[5].clone(), players[8].clone()]),
            ],
            &new_matches_to_play,
        );
        let (_, new_matches_to_play) = both_player_report_match_result(
            &test_api,
            "1",
            "4",
            channel_internal_id,
            service,
            (2, 0),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 0);
        assert_if_matches_exist(&[], &new_matches_to_play);

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[5].get_id().to_string(),
            (2, 0),
            &players[8].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 1);
        assert_if_matches_exist(
            &[([4, 5], [players[4].clone(), players[5].clone()])],
            &new_matches_to_play,
        );

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[4].get_id().to_string(),
            (2, 0),
            &players[5].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 0);
        assert_if_matches_exist(&[], &new_matches_to_play);

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[2].get_id().to_string(),
            (2, 0),
            &players[7].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 0);
        assert_if_matches_exist(&[], &new_matches_to_play);

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[3].get_id().to_string(),
            (2, 0),
            &players[6].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 2);
        assert_if_matches_exist(
            &[
                ([2, 3], [players[2].clone(), players[3].clone()]),
                ([6, 7], [players[6].clone(), players[7].clone()]),
            ],
            &new_matches_to_play,
        );

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[6].get_id().to_string(),
            (2, 0),
            &players[7].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 0);
        assert_if_matches_exist(&[], &new_matches_to_play);

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[2].get_id().to_string(),
            (2, 0),
            &players[3].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 2);
        assert_if_matches_exist(
            &[
                ([1, 2], [players[1].clone(), players[2].clone()]),
                ([3, 6], [players[3].clone(), players[6].clone()]),
            ],
            &new_matches_to_play,
        );

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[3].get_id().to_string(),
            (2, 0),
            &players[6].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 1);
        assert_if_matches_exist(
            &[([3, 4], [players[3].clone(), players[4].clone()])],
            &new_matches_to_play,
        );

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[3].get_id().to_string(),
            (2, 0),
            &players[4].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 0);
        assert_if_matches_exist(&[], &new_matches_to_play);

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[1].get_id().to_string(),
            (2, 0),
            &players[2].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 1);
        assert_if_matches_exist(
            &[([2, 3], [players[2].clone(), players[3].clone()])],
            &new_matches_to_play,
        );

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[2].get_id().to_string(),
            (2, 0),
            &players[3].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 1);
        assert_if_matches_exist(
            &[([1, 2], [players[1].clone(), players[2].clone()])],
            &new_matches_to_play,
        );

        // upset!
        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[1].get_id().to_string(),
            (0, 2),
            &players[2].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 1);
        assert_if_matches_exist(
            &[([1, 2], [players[1].clone(), players[2].clone()])],
            &new_matches_to_play,
        );

        let (_, new_matches_to_play) = tournament_organiser_reports_match_result(
            &test_api,
            channel_internal_id,
            service,
            &players[1].get_id().to_string(),
            (0, 2),
            &players[2].get_id().to_string(),
        )
        .await;
        assert_eq!(new_matches_to_play.len(), 0);
        assert_if_matches_exist(&[], &new_matches_to_play);

        test_api.clean_db().await;
    }
}
