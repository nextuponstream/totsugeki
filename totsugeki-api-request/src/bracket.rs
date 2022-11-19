//! Request to /bracket endpoint

use crate::RequestError;
use crate::HTTP_PREFIX;
use totsugeki::bracket::{
    http_responses::{POSTResult, GET as BracketGET, POST},
    raw::Raw,
    Id as BracketId,
};
use totsugeki::player::{Participants, PlayersRaw, GET as PlayersGET};
use totsugeki::seeding::POST as SeedPOST;

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
                String::new()
            } else {
                format!("/{name}")
            }
        }
        None => String::new(),
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

/// Return player list of active bracket in discussion channel
///
/// # Errors
/// thrown when network is unavailable
pub async fn fetch_players(
    client: reqwest::Client,
    api_url: &str,
    body: PlayersGET,
) -> Result<(BracketId, Participants), RequestError> {
    let res = client
        .get(format!("{HTTP_PREFIX}{api_url}/bracket/players"))
        .json(&body)
        .send()
        .await?;
    let info: PlayersRaw = res.json().await?;
    let players = Participants::from_raw_id(
        info.players
            .into_iter()
            .map(|id| id.to_string())
            .zip(info.player_names.into_iter())
            .collect(),
    )?;
    Ok((info.bracket_id, players))
}

/// Seed active bracket in discussion channel
///
/// # Errors
/// thrown when network is unavailable
pub async fn seed(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
    body: SeedPOST,
) -> Result<BracketId, RequestError> {
    let res = client
        .post(format!("{HTTP_PREFIX}{api_url}/bracket/seed"))
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
