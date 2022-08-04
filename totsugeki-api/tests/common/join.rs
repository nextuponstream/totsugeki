// Joining brackets

use totsugeki::{
    bracket::{Id as BracketId, GET},
    join::POSTRequestBody,
};

use super::{bracket::parse_bracket_get_response, TotsugekiApiTestClient};

/// Make `n` players join a bracket
pub async fn n_players_join_bracket(
    test_api: &TotsugekiApiTestClient,
    channel_internal_id: &str,
    service_type_id: &str,
    bracket_id: BracketId,
) -> GET {
    for i in 1..=3 {
        let player_internal_id = i.to_string();
        let player_name = format!("player_{i}");
        let body = POSTRequestBody::new(
            player_internal_id,
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
    }

    // get bracket details
    let resp = test_api
        .cli
        .get(format!("/bracket/{}", bracket_id))
        .header("X-API-Key", test_api.authorization_header.as_str())
        .send()
        .await;
    resp.assert_status_is_ok();
    parse_bracket_get_response(resp.json().await)
}
