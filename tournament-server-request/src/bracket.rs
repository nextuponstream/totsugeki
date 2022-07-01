//! Request to /bracket endpoint

use crate::RequestError;
use totsugeki::{Bracket, BracketPOST};

// easily switch from http to https
const HTTP_PREFIX: &str = "https://";

/// Create brackets
///
/// # Errors
/// Thrown when server is unavailable or deserialisation could not be made
/// # Panics
/// some reason
pub async fn create(
    client: reqwest::Client,
    tournament_server_url: &str,
    bracket_name: String,
) -> Result<(), RequestError> {
    let body = BracketPOST::new(bracket_name.clone());
    client
        .post(format!("{HTTP_PREFIX}{tournament_server_url}/bracket"))
        .json(&body)
        .send()
        .await?;
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
