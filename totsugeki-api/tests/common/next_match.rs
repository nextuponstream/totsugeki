//! Next match utilites

use super::TotsugekiApiTestClient;
use poem::test::TestJson;
use reqwest::StatusCode;
use totsugeki::{
    bracket::Id as BracketId,
    matches::{Id as MatchId, NextMatchGET, NextMatchGETRequest},
    opponent::Opponent,
    player::Id as PlayerId,
};
use totsugeki_api::Service;
use tracing::trace;

/// Parse response for next match
pub fn parse_next_match_get_response(r: TestJson) -> NextMatchGET {
    let o = r.value().object();
    let opponent = o.get("opponent").string().parse().expect("opponent");
    let match_id = MatchId::parse_str(o.get("match_id").string()).expect("match id");
    let bracket_id = BracketId::parse_str(o.get("bracket_id").string()).expect("bracket id");
    let player_name = o.get("player_name").string().to_string();

    NextMatchGET {
        opponent,
        match_id,
        bracket_id,
        player_name,
    }
}

/// Assert seed `x` player's next opponent is seed `y` player. `players` are sorted by seed
pub async fn assert_next_opponent(
    x: usize,
    y: usize,
    players: &[PlayerId],
    test_api: &TotsugekiApiTestClient,
    channel_internal_id: &str,
    service_type_id: Service,
) {
    let body = NextMatchGETRequest {
        player_internal_id: x.to_string(),
        channel_internal_id: channel_internal_id.to_string(),
        service_type_id: service_type_id.to_string(),
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
        Opponent::Player(*players.get(y - 1).expect("seeded player")),
        "seed {x} is not playing against seed {y}:\n{players:?}"
    );
}

/// Assert seed `x` player's next opponent is unknown. `players` are sorted by seed
pub async fn assert_next_opponent_is_unknown(
    x: usize,
    test_api: &TotsugekiApiTestClient,
    channel_internal_id: &str,
    service_type_id: Service,
) {
    let body = NextMatchGETRequest {
        player_internal_id: x.to_string(),
        channel_internal_id: channel_internal_id.to_string(),
        service_type_id: service_type_id.to_string(),
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
    channel_internal_id: &str,
    service_type_id: Service,
) {
    assert_next_opponent(
        x,
        y,
        players,
        test_api,
        channel_internal_id,
        service_type_id,
    )
    .await;
    assert_next_opponent(
        y,
        x,
        players,
        test_api,
        channel_internal_id,
        service_type_id,
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
    service_type_id: Service,
) {
    for p in players_with_unknown_opponent {
        assert_next_opponent_is_unknown(*p, test_api, &channel_internal_id, service_type_id).await;
    }

    for (p1, p2) in players_playing_each_other {
        assert_player_x_and_y_play_each_other(
            *p1,
            *p2,
            players,
            test_api,
            &channel_internal_id,
            service_type_id,
        )
        .await;
    }
}

/// Assert player with `seed` is eliminated
pub async fn assert_player_is_eliminated(
    test_api: &TotsugekiApiTestClient,
    player: usize,
    channel_internal_id: &str,
    service_type_id: Service,
) {
    trace!("Asserting player is eliminated");
    let body = NextMatchGETRequest {
        player_internal_id: player.to_string(),
        channel_internal_id: channel_internal_id.to_string(),
        service_type_id: service_type_id.to_string(),
    };
    let res = test_api
        .cli
        .get("/next_match")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&body)
        .send()
        .await;
    res.assert_status(StatusCode::NOT_FOUND);
    res.assert_text(
        "Unable to answer query: There is no match for you to play because you were eliminated from the bracket",
    )
    .await;
}

/// Assert player with `seed` has no next match (not enough players or winner)
pub async fn assert_player_has_no_next_match(
    test_api: &TotsugekiApiTestClient,
    player: usize,
    channel_internal_id: &str,
    service_type_id: Service,
) {
    trace!("Asserting player has no next match");
    let body = NextMatchGETRequest {
        player_internal_id: player.to_string(),
        channel_internal_id: channel_internal_id.to_string(),
        service_type_id: service_type_id.to_string(),
    };
    let res = test_api
        .cli
        .get("/next_match")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&body)
        .send()
        .await;
    res.assert_status(StatusCode::NOT_FOUND);
    res.assert_text(
        "Unable to answer query: There is no match for you to play because you won the bracket",
    )
    .await;
}