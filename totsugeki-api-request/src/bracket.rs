//! Request to /bracket endpoint

use crate::RequestError;
use crate::HTTP_PREFIX;
use totsugeki::{Bracket, BracketPOST};

/// Create brackets
///
/// # Errors
/// Thrown when server is unavailable or deserialisation could not be made
pub async fn create(
    client: reqwest::Client,
    tournament_server_url: &str,
    authorization_header: &str,
    bracket_name: &str,
) -> Result<(), RequestError> {
    let body = BracketPOST::new(bracket_name.to_string());
    let res = client
        .post(format!("{HTTP_PREFIX}{tournament_server_url}/bracket"))
        .header("X-API-Key", authorization_header)
        .json(&body)
        .send()
        .await?;
    res.error_for_status()?;
    Ok(())
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
