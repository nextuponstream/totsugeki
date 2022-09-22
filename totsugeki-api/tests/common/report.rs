//! Report match results

use crate::common::bracket::parse_matches;

use super::TotsugekiApiTestClient;
use totsugeki::matches::{Id as MatchId, Match};
use totsugeki::matches::{
    PlayerMatchResultPOST, ReportResultPOST, ReportedResult, TournamentOrganiserMatchResultPOST,
};
use totsugeki_api::Service;
use tracing::debug;

/// player report his `result` from a discussion channel
pub async fn player_reports_match_result(
    test_api: &TotsugekiApiTestClient,
    player_internal_id: &str,
    channel_internal_id: &str,
    service: Service,
    result: ReportedResult,
) -> (ReportResultPOST, Vec<Match>) {
    let body = PlayerMatchResultPOST {
        internal_player_id: player_internal_id.to_string(),
        internal_channel_id: channel_internal_id.to_string(),
        service: service.to_string(),
        result: result.to_string(),
    };
    let res = test_api
        .cli
        .post("/bracket/report/player")
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
    let new_matches_to_play = parse_matches(&r);
    let matches = new_matches_to_play
        .iter()
        .map(|m| m.clone().into())
        .collect();
    (
        ReportResultPOST {
            affected_match_id,
            message,
            matches,
        },
        new_matches_to_play,
    )
}

/// Both player report their results and returns match id.
pub async fn both_player_report_match_result(
    test_api: &TotsugekiApiTestClient,
    player_internal_id_1: &str,
    player_internal_id_2: &str,
    channel_internal_id: &str,
    service: Service,
    reported_result: ReportedResult,
) -> (MatchId, Vec<Match>) {
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
        resp_1.0.affected_match_id, resp_2.0.affected_match_id,
        "players with seed {} and {} are not playing the same match",
        player_internal_id_1, player_internal_id_2
    );
    debug!("Reported results OK by {player_internal_id_1} and {player_internal_id_2}");

    (resp_1.0.affected_match_id, resp_2.1)
}

/// Tournament organiser reports match result
pub async fn tournament_organiser_reports_match_result(
    test_api: &TotsugekiApiTestClient,
    channel_internal_id: &str,
    service: Service,
    player1: &str,
    reported_result: (i8, i8),
    player2: &str,
) -> (MatchId, Vec<Match>) {
    let body = TournamentOrganiserMatchResultPOST {
        internal_channel_id: channel_internal_id.into(),
        service: service.to_string(),
        player1: player1.into(),
        result: ReportedResult(reported_result).to_string(),
        player2: player2.into(),
    };
    let res = test_api
        .cli
        .post("/bracket/report/tournament_organiser")
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
    let _message = r.get("message").string().to_string();
    let new_matches_to_play = parse_matches(&r);

    (affected_match_id, new_matches_to_play)
}
