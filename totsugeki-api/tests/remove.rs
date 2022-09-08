//! Test /bracket/remove

pub mod common;

use chrono::prelude::*;
use common::{
    bracket::{create_bracket, get_bracket, players_join_new_bracket_and_bracket_starts},
    db_types_to_test,
    join::{n_players_join_bracket, player_join_bracket},
    test_api,
};
use reqwest::StatusCode;
use test_log::test;
use totsugeki::{format::Format, remove::POST as RemovePOST, seeding::Method};
use totsugeki_api::Service;

#[test(tokio::test)]
async fn to_remove_player_from_bracket() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let internal_channel_id = "1";
        let service = Service::Discord;
        let (bracket, _, _) = create_bracket(
            &test_api,
            "1",
            internal_channel_id,
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
            internal_channel_id,
            service,
            bracket.bracket_id,
        )
        .await;
        assert_eq!(bracket.players.len(), 5);
        assert!(
            bracket.players.iter().any(|p| p.get_name() == *"player_1"),
            "expected player 1"
        );
        assert_eq!(bracket.matches.len(), 4);
        let player_1 = bracket
            .players
            .iter()
            .find(|p| p.get_name() == "player_1")
            .expect("player 1");

        let resp = test_api
            .cli
            .post("/bracket/remove")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&RemovePOST {
                internal_channel_id: internal_channel_id.into(),
                player_id: player_1.get_id().to_string(),
                service: Service::Discord.to_string(),
            })
            .send()
            .await;
        resp.assert_status_is_ok();

        let bracket = get_bracket(&test_api, bracket.bracket_id).await;
        assert_eq!(
            bracket.players.len(),
            4,
            "Expected 4 players. Players: {:?}",
            bracket.players
        );
        assert!(
            !bracket.players.iter().any(|p| p.get_name() == *"player_1"),
            "player 1 is present"
        );
        assert_eq!(bracket.matches.len(), 3);

        // Player can rejoin
        let bracket = player_join_bracket(&test_api, 1, "1", service, bracket.bracket_id).await;
        assert_eq!(bracket.players.len(), 5);
        assert!(
            bracket.players.iter().any(|p| p.get_name() == *"player_1"),
            "expected player 1"
        );
        assert_eq!(bracket.matches.len(), 4);

        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn cannot_remove_player_after_bracket_starts() {
    for db_type in db_types_to_test() {
        let internal_channel_id = "1";
        let test_api = test_api(db_type).await;
        let (bracket, _) = players_join_new_bracket_and_bracket_starts(
            &test_api,
            "1",
            internal_channel_id,
            Service::Discord,
            Format::default(),
            Method::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            5,
            false,
        )
        .await;
        let player_1 = bracket
            .players
            .iter()
            .find(|p| p.get_name() == "player_1")
            .expect("player 1");

        let resp = test_api
            .cli
            .post("/bracket/remove")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&RemovePOST {
                internal_channel_id: internal_channel_id.into(),
                player_id: player_1.get_id().to_string(),
                service: Service::Discord.to_string(),
            })
            .send()
            .await;
        resp.assert_status(StatusCode::FORBIDDEN);
        resp.assert_text(format!("Action is forbidden:\n\tBracket {} has started. As a player, you can quit the bracket by forfeiting or ask an admin to disqualify you.", bracket.bracket_id)).await;

        test_api.clean_db().await;
    }
}
