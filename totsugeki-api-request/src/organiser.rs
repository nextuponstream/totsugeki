//! Request to /organiser endpoint

use crate::{RequestError, HTTP_PREFIX};
use totsugeki::organiser::Organiser;

/// Fetch organisers
///
/// # Errors
/// Returns an error when the api is unavailable
pub async fn fetch(
    client: reqwest::Client,
    api_url: &str,
    organiser_name_filter: Option<String>,
    offset: i64,
) -> Result<Vec<Organiser>, RequestError> {
    let filter = match organiser_name_filter {
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
        .get(format!("{HTTP_PREFIX}{api_url}/organiser{filter}/{offset}"))
        .send()
        .await?;
    let organisers: Vec<Organiser> = res.json().await?;
    Ok(organisers)
}
