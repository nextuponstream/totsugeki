//! Request to /bracket endpoint

use crate::RequestError;
use crate::HTTP_PREFIX;
use totsugeki::bracket::{Id as BracketId, POSTResult, Raw, GET as BracketGET, POST};

/// Create brackets
///
/// # Errors
/// Thrown when server is unavailable or deserialisation could not be made
pub async fn create<'a>(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
    body: POST,
) -> Result<POSTResult, RequestError> {
    let res = client
        .post(format!("{HTTP_PREFIX}{api_url}/bracket"))
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
    api_url: &str,
    bracket_name_filter: Option<String>,
    offset: i64,
) -> Result<Vec<Raw>, RequestError> {
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
        .get(format!("{HTTP_PREFIX}{api_url}/brackets{filter}/{offset}"))
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
    api_url: &str,
    bracket_id: BracketId,
) -> Result<Raw, RequestError> {
    let res = client
        .get(format!("{HTTP_PREFIX}{api_url}/bracket/{bracket_id}"))
        .send()
        .await?;
    let bracket: BracketGET = res.json().await?;
    Ok(bracket.try_into()?)
}
