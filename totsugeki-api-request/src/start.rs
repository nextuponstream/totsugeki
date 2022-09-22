//! Start bracket and accept match results

use crate::{RequestError, HTTP_PREFIX};
use totsugeki::{
    bracket::http_responses::{CommandPOST, StartPOSTResult},
    DiscussionChannel,
};

/// Start bracket in discussion channel
///
/// # Errors
/// Thrown when request could not be processed by the server
pub async fn post<T: DiscussionChannel>(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
    discussion_channel: T,
) -> Result<StartPOSTResult, RequestError> {
    let body = CommandPOST {
        channel_internal_id: discussion_channel.get_internal_id().to_string(),
        service_type_id: discussion_channel.get_service_type(),
    };
    let res = client
        .post(format!("{HTTP_PREFIX}{api_url}/bracket/start"))
        .header("X-API-Key", authorization_header)
        .json(&body)
        .send()
        .await?;
    // use _ref so res is not consumed
    match res.error_for_status_ref() {
        Ok(_) => {
            let response = res.json::<StartPOSTResult>().await?;
            Ok(response)
        }
        Err(r) => {
            let txt = res.text().await?;
            Err(RequestError::Request(r, txt))
        }
    }
}
