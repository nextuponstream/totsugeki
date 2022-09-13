// Joining brackets

use totsugeki::{
    bracket::{http_responses::GET, Id as BracketId},
    join::POST,
};
use totsugeki_api::Service;

use super::{bracket::get_bracket, TotsugekiApiTestClient};

/// Player with `id` joins `bracket_id`. Their name is "player_{id}"
pub async fn player_join_bracket(
    test_api: &TotsugekiApiTestClient,
    internal_player_id: usize,
    channel_internal_id: &str,
    service_type_id: Service,
    bracket_id: BracketId,
) -> GET {
    let player_name = format!("player_{internal_player_id}");
    let body = POST::new(
        internal_player_id.to_string(),
        player_name,
        channel_internal_id.to_string(),
        service_type_id.to_string(),
    );

    let resp = test_api
        .cli
        .post("/join")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&body)
        .send()
        .await;
    resp.assert_status_is_ok();
    get_bracket(test_api, bracket_id).await
}

/// Make `n` players join a bracket. Returns affected bracket after players
/// joined.
///
/// First player to join has internal id 1 and is named "player_1", and so on
pub async fn n_players_join_bracket(
    test_api: &TotsugekiApiTestClient,
    n: usize,
    channel_internal_id: &str,
    service_type_id: Service,
    bracket_id: BracketId,
) -> GET {
    for i in 1..=n {
        player_join_bracket(
            test_api,
            i,
            channel_internal_id,
            service_type_id,
            bracket_id,
        )
        .await;
    }

    // get bracket details
    get_bracket(test_api, bracket_id).await
}
