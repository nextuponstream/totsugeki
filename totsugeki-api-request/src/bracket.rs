//! Request to /bracket endpoint

use crate::RequestError;
use crate::HTTP_PREFIX;
use totsugeki::bracket::RequestParameters;
use totsugeki::bracket::{Bracket, Id as BracketId, POSTResult, GET as BracketGET, POST};
use totsugeki::DiscussionChannel;

/// Create brackets
///
/// # Errors
/// Thrown when server is unavailable or deserialisation could not be made
pub async fn create<'a, T: DiscussionChannel>(
    client: reqwest::Client,
    tournament_server_url: &str,
    authorization_header: &str,
    p: RequestParameters<'a, T>,
) -> Result<POSTResult, RequestError> {
    let body = POST::new(
        p.bracket_name.to_string(),
        p.organiser_name.to_string(),
        p.organiser_id.to_string(),
        p.discussion_channel.get_internal_id().to_string(),
        p.discussion_channel.get_service_type(),
        p.bracket_format.to_string(),
        p.seeding_method.to_string(),
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
            "{HTTP_PREFIX}{tournament_server_url}/brackets{filter}/{offset}"
        ))
        .send()
        .await?;
    let text: Vec<BracketGET> = res.json().await?;
    let mut brackets = vec![];
    for b in text {
        brackets.push(b.try_into()?);
    }
    Ok(brackets)
}

/// Get bracket from id
///
/// # Errors
/// Thrown when server is unavailable or deserialisation could not be made
pub async fn get_from_id(
    client: reqwest::Client,
    tournament_server_url: &str,
    bracket_id: BracketId,
) -> Result<Bracket, RequestError> {
    let res = client
        .get(format!(
            "{HTTP_PREFIX}{tournament_server_url}/bracket/{bracket_id}"
        ))
        .send()
        .await?;
    let bracket: BracketGET = res.json().await?;
    Ok(bracket.try_into()?)
}
