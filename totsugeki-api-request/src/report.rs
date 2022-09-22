//! report match result

use crate::{RequestError, HTTP_PREFIX};
use totsugeki::{
    matches::{PlayerMatchResultPOST, ReportResultPOST, TournamentOrganiserMatchResultPOST},
    DiscussionChannel,
};

/// Player reports result of match using discussion channel where message is
/// written
///
/// # Errors
/// Returns error if the api returns an invalid Id for affected match
pub async fn player_reports_result<T: DiscussionChannel>(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
    player_internal_id: &str,
    result: &str,
    discussion_channel: T,
) -> Result<ReportResultPOST, RequestError> {
    let body = PlayerMatchResultPOST {
        internal_player_id: player_internal_id.to_string(),
        internal_channel_id: discussion_channel.get_internal_id().to_string(),
        service: discussion_channel.get_service_type(),
        result: result.to_string(),
    };
    let res = client
        .post(format!("{HTTP_PREFIX}{api_url}/bracket/report/player"))
        .header("X-API-Key", authorization_header)
        .json(&body)
        .send()
        .await?;

    match res.error_for_status_ref() {
        Ok(_) => {
            let response: ReportResultPOST = res.json().await?;
            Ok(response)
        }
        Err(e) => Err(RequestError::Request(e, res.text().await?)),
    }
}

/// Tournament organiser reports result of match using discussion channel where
/// bracket is active
///
/// # Errors
/// Returns error if the api returns an invalid Id for affected match
pub async fn tournament_organiser_reports_result<T: DiscussionChannel>(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
    player1: &str,
    result: &str,
    player2: &str,
    discussion_channel: T,
) -> Result<ReportResultPOST, RequestError> {
    let body = TournamentOrganiserMatchResultPOST {
        internal_channel_id: discussion_channel.get_internal_id().to_string(),
        service: discussion_channel.get_service_type(),
        player1: player1.to_string(),
        result: result.to_string(),
        player2: player2.to_string(),
    };
    let res = client
        .post(format!(
            "{HTTP_PREFIX}{api_url}/bracket/report/tournament_organiser"
        ))
        .header("X-API-Key", authorization_header)
        .json(&body)
        .send()
        .await?;

    match res.error_for_status_ref() {
        Ok(_) => {
            let response: ReportResultPOST = res.json().await?;
            Ok(response)
        }
        Err(e) => Err(RequestError::Request(e, res.text().await?)),
    }
}
