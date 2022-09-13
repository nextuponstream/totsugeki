//! Disqualify players test utils

use totsugeki::{
    bracket::{http_responses::GET, Id as BracketId},
    player::Id as PlayerId,
    remove::POST,
};
use totsugeki_api::Service;

use super::{bracket::get_bracket, TotsugekiApiTestClient};

/// Disqualify player with internal id `x`
pub async fn organiser_disqualify_player_from_bracket(
    test_api: &TotsugekiApiTestClient,
    player_id: PlayerId,
    internal_channel_id: &str,
    service: Service,
    bracket_id: BracketId,
) -> GET {
    let resp = test_api
        .cli
        .post("/bracket/disqualify")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&POST {
            internal_channel_id: internal_channel_id.into(),
            player_id: player_id.to_string(),
            service: service.to_string(),
        })
        .send()
        .await;
    resp.assert_status_is_ok();
    get_bracket(test_api, bracket_id).await
}
