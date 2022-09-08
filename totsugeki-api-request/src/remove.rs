//! As TO, remove player from bracket
use crate::{RequestError, HTTP_PREFIX};
use totsugeki::{bracket::Id as BracketId, remove::POST};

/// Remove player from bracket
///
/// # Errors
/// thrown when there is an error with the network
pub async fn post(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
    body: POST,
) -> Result<BracketId, RequestError> {
    let res = client
        .post(format!("{HTTP_PREFIX}{api_url}/bracket/remove"))
        .header("X-API-Key", authorization_header)
        .json(&body)
        .send()
        .await?;

    match res.error_for_status_ref() {
        Ok(_) => {
            let mut response = res.text().await?;
            response.pop();
            response.remove(0);
            let bracket_id = BracketId::parse_str(response.as_str())?;
            Ok(bracket_id)
        }
        Err(e) => Err(RequestError::Request(e, res.text().await?)),
    }
}
