// test /bracket/start endpoint

pub mod common;

use crate::common::join::n_players_join_bracket;
use chrono::prelude::*;
use common::{
    bracket::{create_bracket, parse_matches},
    db_types_to_test, test_api,
};
use reqwest::StatusCode;
use test_log::test;
use totsugeki::{bracket::http_responses::CommandPOST, format::Format, seeding::Method};
use totsugeki_api::Service;

#[test(tokio::test)]
async fn matches_to_play_at_bracket_start_for_5_participants() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let channel_internal_id = "1";
        let service = Service::Discord;
        let (bracket, _, _) = create_bracket(
            &test_api,
            "1",
            channel_internal_id,
            service,
            Format::SingleElimination,
            Method::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        )
        .await;
        let bracket = n_players_join_bracket(
            &test_api,
            5,
            channel_internal_id,
            service,
            bracket.bracket_id,
        )
        .await;

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
        assert_eq!(matches_to_play.len(), 2);
        assert!(matches_to_play.iter().any(|m| m.get_seeds() == [4, 5]));
        assert!(matches_to_play.iter().any(|m| m.get_seeds() == [2, 3]));
        assert!(matches_to_play
            .iter()
            .any(|m| !m.contains(bracket.players.get(0).expect("p1").get_id())));
        assert!(matches_to_play
            .iter()
            .any(|m| m.contains(bracket.players.get(1).expect("p2").get_id())));
        assert!(matches_to_play
            .iter()
            .any(|m| m.contains(bracket.players.get(2).expect("p3").get_id())));
        assert!(matches_to_play
            .iter()
            .any(|m| m.contains(bracket.players.get(3).expect("p4").get_id())));
        assert!(matches_to_play
            .iter()
            .any(|m| m.contains(bracket.players.get(4).expect("p5").get_id())));

        test_api.clean_db().await;
    }
}
