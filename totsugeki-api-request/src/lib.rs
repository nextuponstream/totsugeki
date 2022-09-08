#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

//! All requests made by external services to the api
//!
//! **Note**: reqwest `danger_accept_invalid_certs` method does not compile to wasm target
//! which is why reqwest client is passed as a parameter. frontend is the only crate compiling to wasm
//! but discord bot makes the same request. This parameter being added to the parameter list is a necessary code smell.
use thiserror::Error;
use totsugeki::{
    bracket::ParsingError as BracketParsingError, matches::NextMatchGETParsingError,
    player::Error as PlayerError, ServiceRegisterPOST,
};

pub mod bracket;
pub mod close;
pub mod join;
pub mod next_match;
pub mod organiser;
pub mod quit;
pub mod remove;
pub mod report;
pub mod start;
pub mod validate;

/// Helper for forming url
const HTTP_PREFIX: &str = "https://";

/// Error while making request to the api
#[derive(Error, Debug)]
pub enum RequestError {
    /// Request error
    #[error("{1}")]
    Request(reqwest::Error, String),
    /// Bracket parsing error
    #[error("{0}")]
    BracketParsingError(#[from] BracketParsingError),
    /// Match id parsing error
    #[error("Could not parse match id: {0}")]
    MatchIdParsingError(#[from] uuid::Error),
    /// Error parsing next match
    #[error("{0}")]
    NextMatch(#[from] NextMatchGETParsingError),
    /// Cannot parse players in response
    #[error("{0}")]
    PlayerParsingError(#[from] PlayerError),
}

impl From<reqwest::Error> for RequestError {
    fn from(e: reqwest::Error) -> Self {
        let msg = e.to_string();
        let status = match e.status() {
            Some(s) => format!("({s})"),
            None => String::new(),
        };
        RequestError::Request(e, format!("Request to Api has failed: {}{}", status, msg))
    }
}

/// Use API endpoint to clean database for test purposes
///
/// # Errors
/// Returns an error when the api is unavailable
pub async fn clean_database(
    client: reqwest::Client,
    api_url: &str,
    authorization_header: &str,
) -> Result<(), RequestError> {
    let res = client
        .delete(format!("{HTTP_PREFIX}{api_url}/clean"))
        .header("X-API-Key", authorization_header)
        .send()
        .await?;
    res.error_for_status()?;
    Ok(())
}

/// Get service API token for tests
///
/// # Errors
/// Returns an error when the api is unavailable
pub async fn get_service_token(
    client: reqwest::Client,
    api_url: &str,
) -> Result<ServiceRegisterPOST, RequestError> {
    let res = client
        .post(format!(
            "{HTTP_PREFIX}{api_url}/service/register/test-service/for-tests"
        ))
        .send()
        .await?;
    let token = res.json::<ServiceRegisterPOST>().await?;
    Ok(token)
}
