//! report match result

use crate::{RequestError, HTTP_PREFIX};
use totsugeki::{
    matches::{Id as MatchId, MatchResultPOST},
    DiscussionChannel,
};

/// Report result of match using discussion channel where message is written
///
/// # Errors
/// Returns error if the api returns an invalid Id for affected match
pub async fn result<T: DiscussionChannel>(
    client: reqwest::Client,
    tournament_server_url: &str,
    authorization_header: &str,
    player_internal_id: &str,
    result: &str,
    discussion_channel: T,
) -> Result<MatchId, RequestError> {
    let body = MatchResultPOST {
        player_internal_id: player_internal_id.to_string(),
        channel_internal_id: discussion_channel.get_internal_id().to_string(),
        service_type_id: discussion_channel.get_service_type(),
        result: result.to_string(),
    };
    let res = client
        .post(format!(
            "{HTTP_PREFIX}{tournament_server_url}/bracket/report"
        ))
        .header("X-API-Key", authorization_header)
        .json(&body)
        .send()
        .await?;
    // use _ref so res is not consumed
    match res.error_for_status_ref() {
        Ok(_) => {
            let mut response = res.text().await?;
            response.pop();
            response.remove(0);
            let match_id = MatchId::parse_str(response.as_str())?;
            Ok(match_id)
        }
        Err(r) => {
            let txt = res.text().await?;
            Err(RequestError::Request(r, txt))
        }
    }
}
