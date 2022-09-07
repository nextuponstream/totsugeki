//! Common utilities for parsing players in test objects

use super::TotsugekiApiTestClient;
use poem::test::TestJsonObject;
use totsugeki::{
    bracket::Id as BracketId,
    player::{Participants, GET},
};

pub async fn query_players(
    test_api: &TotsugekiApiTestClient,
    body: &GET,
) -> (BracketId, Participants) {
    let resp = test_api
        .cli
        .get("/bracket/players".to_string())
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

pub fn parse_players(r: &TestJsonObject) -> Participants {
    Participants::from_raw_id(
        r.get("players")
            .string_array()
            .iter()
            .map(|p| p.to_string())
            .zip(
                r.get("player_names")
                    .string_array()
                    .iter()
                    .map(|n| n.to_string()),
            )
            .collect(),
    )
    .expect("player group")
}
