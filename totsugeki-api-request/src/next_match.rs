//! Get next match to play

use crate::{RequestError, HTTP_PREFIX};
use totsugeki::{
    matches::{NextMatchGET as GET, NextMatchGETRequest, NextMatchGETResponse as GETRaw},
    DiscussionChannel,
};

/// Get next match to play
///
/// # Errors
/// Returns error if there is an error with the network
pub async fn next_match<T: DiscussionChannel>(
    client: reqwest::Client,
    tournament_server_url: &str,
    authorization_header: &str,
    player_internal_id: &str,
    discussion_channel: T,
) -> Result<GET, RequestError> {
    let body = NextMatchGETRequest {
        player_internal_id: player_internal_id.to_string(),
        channel_internal_id: discussion_channel.get_internal_id().to_string(),
        service_type_id: discussion_channel.get_service_type(),
    };
    let res = client
        .get(format!("{HTTP_PREFIX}{tournament_server_url}/next_match",))
        .header("X-API-Key", authorization_header)
        .json(&body)
        .send()
        .await?;

    match res.error_for_status_ref() {
        Ok(_) => {
            let next_match: GETRaw = res.json().await?;
            let next_match = GET::try_from(next_match)?;
            Ok(next_match)
        }
        Err(e) => Err(RequestError::Request(e, res.text().await?)),
    }
}
