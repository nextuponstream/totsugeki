//! Report match results

use super::TotsugekiApiTestClient;
use totsugeki::matches::Id as MatchId;
use totsugeki::matches::{MatchResultPOST, ReportResultPOST, ReportedResult};
use totsugeki_api::Service;
use tracing::debug;

/// player report his `result` from a discussion channel
pub async fn player_reports_match_result(
    test_api: &TotsugekiApiTestClient,
    player_internal_id: &str,
    channel_internal_id: &str,
    service: Service,
    result: ReportedResult,
) -> ReportResultPOST {
    let body = MatchResultPOST {
        internal_player_id: player_internal_id.to_string(),
        internal_channel_id: channel_internal_id.to_string(),
        service: service.to_string(),
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
    let response = res.json().await;
    let r = response.value().object();
    let affected_match_id =
        MatchId::parse_str(r.get("affected_match_id").string()).expect("affected match id");
    println!("{r:?}");
    let message = r.get("message").string().to_string();
    ReportResultPOST {
        affected_match_id,
        message,
    }
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
    debug!("Reporting match for player {player_internal_id_1} and {player_internal_id_2}");
    let resp_1 = player_reports_match_result(
        test_api,
        player_internal_id_1,
        channel_internal_id,
        service,
        reported_result,
    )
    .await;
    let resp_2 = player_reports_match_result(
        test_api,
        player_internal_id_2,
        channel_internal_id,
        service,
        reported_result.reverse(),
    )
    .await;

    assert_eq!(
        resp_1.affected_match_id, resp_2.affected_match_id,
        "players with seed {} and {} are not playing the same match",
        player_internal_id_1, player_internal_id_2
    );
    debug!("Reported results OK by {player_internal_id_1} and {player_internal_id_2}");

    resp_1.affected_match_id
}
