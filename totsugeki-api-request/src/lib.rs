#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

//! All requests made by external services to the tournament server
//!
//! **Note**: reqwest `danger_accept_invalid_certs` method does not compile to wasm target
//! which is why reqwest client is passed as a parameter. frontend is the only crate compiling to wasm
//! but discord bot makes the same request. This parameter being added to the parameter list is a necessary code smell.
use std::fmt::{self, Formatter};
use totsugeki::ServiceRegisterPOST;

pub mod bracket;
pub mod join;
pub mod organiser;

// easily switch from http to https
const HTTP_PREFIX: &str = "https://";

/// Error while making request to the tournament server
#[derive(Debug)]
pub enum RequestError {
    /// Request error
    Request(reqwest::Error),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::Request(e) => e.fmt(f),
        }
    }
}

impl From<reqwest::Error> for RequestError {
    fn from(e: reqwest::Error) -> Self {
        RequestError::Request(e)
    }
}

impl std::error::Error for RequestError {}

/// Use API endpoint to clean database for test purposes.
pub async fn clean_database(
    client: reqwest::Client,
    tournament_server_url: &str,
    authorization_header: &str,
) -> Result<(), RequestError> {
    let res = client
        .delete(format!("{HTTP_PREFIX}{tournament_server_url}/clean"))
        .header("X-API-Key", authorization_header)
        .send()
        .await?;
    res.error_for_status()?;
    Ok(())
}

/// Get service API token for tests
pub async fn get_service_token(
    client: reqwest::Client,
    tournament_server_url: &str,
) -> Result<ServiceRegisterPOST, RequestError> {
    let res = client
        .post(format!(
            "{HTTP_PREFIX}{tournament_server_url}/service/register/test-service/for-tests"
        ))
        .send()
        .await?;
    let token = res.json::<ServiceRegisterPOST>().await?;
    Ok(token)
}
