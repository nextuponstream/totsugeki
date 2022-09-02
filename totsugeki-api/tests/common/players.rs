//! Common utilities for parsing players in test objects

use super::TotsugekiApiTestClient;
use poem::test::TestJsonObject;
use totsugeki::{
    bracket::Id as BracketId,
    player::{Players, GET},
};

pub async fn query_players(test_api: &TotsugekiApiTestClient, body: &GET) -> (BracketId, Players) {
    let resp = test_api
        .cli
        .get(format!("/bracket/players"))
        .body_json(body)
        .send()
        .await;
    resp.assert_status_is_ok();
    let resp = resp.json().await;
    let r = resp.value().object();
    let bracket_id = r
        .get("bracket_id")
        .string()
        .parse::<BracketId>()
        .expect("bracket id");
    let players = parse_players(&r);
    (bracket_id, players)
}

pub fn parse_players(r: &TestJsonObject) -> Players {
    let players: Vec<String> = r
        .get("players")
        .string_array()
        .iter()
        .map(|p| p.to_string())
        .collect();
    let player_names = r
        .get("player_names")
        .string_array()
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<String>>();
    Players::from_raw_id(players.into_iter().zip(player_names.into_iter()).collect())
        .expect("player group")
}
