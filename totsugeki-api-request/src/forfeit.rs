//! Forfeit in bracket

use crate::{RequestError, HTTP_PREFIX};
use totsugeki::{bracket::Id as BracketId, quit::POST, DiscussionChannel};

/// Allow participant to forfeit in bracket in case they something comes up
///
/// # Errors
/// thrown if there is no active bracket in discussion channel
pub async fn post<T: DiscussionChannel>(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
    discussion_channel: T,
    user_internal_id: &str,
) -> Result<BracketId, RequestError> {
    let body = POST {
        internal_channel_id: discussion_channel.get_internal_id().to_string(),
        service: discussion_channel.get_service_type(),
        internal_player_id: user_internal_id.into(),
    };
    let res = client
        .post(format!("{HTTP_PREFIX}{api_url}/bracket/forfeit"))
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
            let bracket_id = BracketId::parse_str(response.as_str())?;
            Ok(bracket_id)
        }
        Err(r) => {
            let txt = res.text().await?;
            Err(RequestError::Request(r, txt))
        }
    }
}
