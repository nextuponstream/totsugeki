//! report match result

use crate::{RequestError, HTTP_PREFIX};
use totsugeki::{
    matches::{MatchResultPOST, ReportResultPOST},
    DiscussionChannel,
};

/// Report result of match using discussion channel where message is written
///
/// # Errors
/// Returns error if the api returns an invalid Id for affected match
pub async fn result<T: DiscussionChannel>(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
    player_internal_id: &str,
    result: &str,
    discussion_channel: T,
) -> Result<ReportResultPOST, RequestError> {
    let body = MatchResultPOST {
        internal_player_id: player_internal_id.to_string(),
        internal_channel_id: discussion_channel.get_internal_id().to_string(),
        service: discussion_channel.get_service_type(),
        result: result.to_string(),
    };
    let res = client
        .post(format!("{HTTP_PREFIX}{api_url}/bracket/report"))
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
