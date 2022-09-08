//! Request to /join endpoint

use crate::RequestError;
use crate::HTTP_PREFIX;
use totsugeki::{
    join::{POSTResponse, POST},
    DiscussionChannel,
};

/// Join bracket as a player
///
/// # Errors
/// Thrown when request could not be processed by the server
pub async fn post<T: DiscussionChannel>(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
    player_internal_id: &str,
    player_name: &str,
    discussion_channel: T,
) -> Result<POSTResponse, RequestError> {
    let body = POST::new(
        player_internal_id.to_string(),
        player_name.to_string(),
        discussion_channel.get_internal_id().to_string(),
        discussion_channel.get_service_type(),
    );
    let res = client
        .post(format!("{HTTP_PREFIX}{api_url}/join"))
        .header("X-API-Key", authorization_header)
        .json(&body)
        .send()
        .await?;
    // use _ref so res is not consumed
    match res.error_for_status_ref() {
        Ok(_) => {
            let response = res.json::<POSTResponse>().await?;
            Ok(response)
        }
        Err(r) => {
            let txt = res.text().await?;
            Err(RequestError::Request(r, txt))
        }
    }
}
