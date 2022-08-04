//! Report match results

use totsugeki::matches::MatchResultPOST;

use super::TotsugekiApiTestClient;

/// player report his `result` from a discussion channel
pub async fn player_reports_match_result(
    test_api: &TotsugekiApiTestClient,
    player_internal_id: &str,
    channel_internal_id: &str,
    service_type_id: &str,
    result: &str,
) {
    let body = MatchResultPOST {
        player_internal_id: player_internal_id.to_string(),
        channel_internal_id: channel_internal_id.to_string(),
        service_type_id: service_type_id.to_string(),
        result: result.to_string(),
    };
    let res = test_api
        .cli
        .post("/bracket/report")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&body)
        .send()
        .await;
    res.assert_status_is_ok();
}
