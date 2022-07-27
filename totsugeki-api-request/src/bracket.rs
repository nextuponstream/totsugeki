//! Request to /bracket endpoint

use crate::RequestError;
use crate::HTTP_PREFIX;
use totsugeki::bracket::{Bracket, POSTResult, POST};
use totsugeki::DiscussionChannel;

/// Create brackets
///
/// # Errors
/// Thrown when server is unavailable or deserialisation could not be made
pub async fn create<T: DiscussionChannel>(
    client: reqwest::Client,
    tournament_server_url: &str,
    authorization_header: &str,
    bracket_name: &str,
    organiser_name: &str,
    organiser_id: &str,
    discussion_channel: T,
) -> Result<POSTResult, RequestError> {
    let body = POST::new(
        bracket_name.to_string(),
        organiser_name.to_string(),
        organiser_id.to_string(),
        discussion_channel.get_internal_id().to_string(),
        discussion_channel.get_service_type(),
    );
    let res = client
        .post(format!("{HTTP_PREFIX}{tournament_server_url}/bracket"))
        .header("X-API-Key", authorization_header)
        .json(&body)
        .send()
        .await?;
    let ids: POSTResult = res.json::<POSTResult>().await?;
    Ok(ids)
}

/// Fetch brackets and filter results by `bracket_name_filter`
///
/// # Errors
/// Thrown when server is unavailable or deserialisation could not be made
pub async fn fetch(
    client: reqwest::Client,
    tournament_server_url: &str,
    bracket_name_filter: Option<String>,
    offset: i64,
) -> Result<Vec<Bracket>, RequestError> {
    let filter = match bracket_name_filter {
        Some(name) => {
            if name.is_empty() {
                "".to_string()
            } else {
                format!("/{name}")
            }
        }
        None => "".to_string(),
    };
    let res = client
        .get(format!(
            "{HTTP_PREFIX}{tournament_server_url}/bracket{filter}/{offset}"
        ))
        .send()
        .await?;
    let text: Vec<Bracket> = res.json().await?;
    Ok(text)
}
