//! Report match results

use totsugeki::matches::Id as MatchId;
use totsugeki::matches::{MatchResultPOST, ReportedResult};
use totsugeki_api::Service;

use super::TotsugekiApiTestClient;

/// player report his `result` from a discussion channel
pub async fn player_reports_match_result(
    test_api: &TotsugekiApiTestClient,
    player_internal_id: &str,
    channel_internal_id: &str,
    service: Service,
    result: ReportedResult,
) -> MatchId {
    let body = MatchResultPOST {
        player_internal_id: player_internal_id.to_string(),
        channel_internal_id: channel_internal_id.to_string(),
        service_type_id: service.to_string(),
        result: result.to_string(),
    };
    let mut res = test_api
        .cli
        .post("/bracket/report")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&body)
        .send()
        .await;
    res.assert_status_is_ok();
    let mut response = res.0.take_body().into_string().await.expect("response");
    response.pop();
    response.remove(0);
    MatchId::parse_str(response.as_str()).expect("match id")
}

/// Both player report their results and returns match id.
pub async fn both_player_report_match_result(
    test_api: &TotsugekiApiTestClient,
    player_internal_id_1: &str,
    player_internal_id_2: &str,
    channel_internal_id: &str,
    service: Service,
    reported_result: ReportedResult,
) -> MatchId {
    let match_id_1 = player_reports_match_result(
        test_api,
        player_internal_id_1,
        channel_internal_id,
        service,
        reported_result,
    )
    .await;
    let match_id_2 = player_reports_match_result(
        test_api,
        player_internal_id_2,
        channel_internal_id,
        service,
        reported_result.reverse(),
    )
    .await;

    assert_eq!(match_id_1, match_id_2);

    match_id_1
}
