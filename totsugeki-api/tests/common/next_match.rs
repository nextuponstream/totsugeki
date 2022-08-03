//! Next match utilites

use poem::test::TestJson;
use reqwest::StatusCode;
use totsugeki::{
    bracket::Id as BracketId,
    matches::{Id as MatchId, NextMatchGET, Opponent},
    player::Id as PlayerId,
};
use totsugeki_api::matches::NextMatchGETRequest;

use super::TotsugekiApiTestClient;

/// Parse response for next match
pub fn parse_next_match_get_response(r: TestJson) -> NextMatchGET {
    let o = r.value().object();
    let opponent = Opponent::try_from(o.get("opponent").string().to_string()).expect("opponent");
    let match_id = MatchId::parse_str(o.get("match_id").string()).expect("match id");
    let bracket_id = BracketId::parse_str(o.get("bracket_id").string()).expect("bracket id");

    NextMatchGET {
        opponent,
        match_id,
        bracket_id,
    }
}

/// Assert seed `x` player's next opponent is seed `y` player. `players` are sorted by seed
pub async fn assert_next_opponent(
    x: usize,
    y: usize,
    players: &[PlayerId],
    test_api: &TotsugekiApiTestClient,
    channel_internal_id: String,
    service_type_id: String,
) {
    let body = NextMatchGETRequest {
        player_internal_id: x.to_string(),
        channel_internal_id,
        service_type_id,
    };
    let res = test_api
        .cli
        .get("/next_match")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&body)
        .send()
        .await;
    res.assert_status(StatusCode::OK);
    let r = res.json().await;
    let next_match = parse_next_match_get_response(r);
    assert_eq!(
        next_match.opponent,
        Opponent::Player(*players.get(y - 1).expect("seeded player"))
    );
}

/// Assert seed `x` player's next opponent is unknown. `players` are sorted by seed
pub async fn assert_next_opponent_is_unknown(
    x: usize,
    test_api: &TotsugekiApiTestClient,
    channel_internal_id: String,
    service_type_id: String,
) {
    let body = NextMatchGETRequest {
        player_internal_id: x.to_string(),
        channel_internal_id,
        service_type_id,
    };
    let res = test_api
        .cli
        .get("/next_match")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&body)
        .send()
        .await;
    res.assert_status(StatusCode::OK);
    let r = res.json().await;
    let next_match = parse_next_match_get_response(r);
    assert_eq!(next_match.opponent, Opponent::Unknown);
}

/// Assert seed `x` player plays seed `y` player. `players` are sorted by seed
pub async fn assert_player_x_and_y_play_each_other(
    x: usize,
    y: usize,
    players: &[PlayerId],
    test_api: &TotsugekiApiTestClient,
    channel_internal_id: String,
    service_type_id: String,
) {
    assert_next_opponent(
        x,
        y,
        players,
        test_api,
        channel_internal_id.clone(),
        service_type_id.clone(),
    )
    .await;
    assert_next_opponent(
        y,
        x,
        players,
        test_api,
        channel_internal_id.clone(),
        service_type_id.clone(),
    )
    .await;
}

/// Assert who has unknown opponent and who plays each other
pub async fn assert_next_matches(
    players_with_unknown_opponent: &[usize],
    players_playing_each_other: &[(usize, usize)],
    players: &[PlayerId],
    test_api: &TotsugekiApiTestClient,
    channel_internal_id: String,
    service_type_id: String,
) {
    for p in players_with_unknown_opponent {
        assert_next_opponent_is_unknown(
            *p,
            test_api,
            channel_internal_id.clone(),
            service_type_id.clone(),
        )
        .await;
    }

    for (p1, p2) in players_playing_each_other {
        assert_player_x_and_y_play_each_other(
            *p1,
            *p2,
            players,
            test_api,
            channel_internal_id.clone(),
            service_type_id.clone(),
        )
        .await;
    }
}
