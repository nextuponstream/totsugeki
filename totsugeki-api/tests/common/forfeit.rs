//! Forfeiting players test utils

use totsugeki::{
    bracket::{http_responses::GET, Id as BracketId},
    quit::POST,
};
use totsugeki_api::Service;

use super::{bracket::get_bracket, TotsugekiApiTestClient};

/// Player with internal id `x` declares forfeit
pub async fn request(
    test_api: &TotsugekiApiTestClient,
    internal_player_id: &str,
    internal_channel_id: &str,
    service: Service,
    bracket_id: BracketId,
) -> GET {
    let resp = test_api
        .cli
        .post("/bracket/forfeit")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&POST {
            internal_channel_id: internal_channel_id.into(),
            internal_player_id: internal_player_id.into(),
            service: service.to_string(),
        })
        .send()
        .await;
    resp.assert_status_is_ok();
    get_bracket(test_api, bracket_id).await
}
