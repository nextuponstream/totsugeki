//! Bracket management commands
pub mod create;
pub mod find;
pub mod get;

use self::create::CREATE_COMMAND;
use self::find::FIND_COMMAND;
use self::get::GET_COMMAND;
use poem::error::ResponseError;
use serenity::framework::standard::macros::group;
use std::fmt::Display;

#[group]
#[allow(missing_docs)]
#[prefix("bracket")]
#[commands(create, get, find)]
#[summary = "Main available commands"]
#[description = "Subcommand for TO's to manage a bracket"]
pub struct Bracket;

/// Errors while bot issues command
#[derive(Debug)]
pub enum Error {
    /// General errors
    OhNo(String),
}

/// Get client that `accept_invalid_certs` for testing
fn get_client(accept_invalid_certs: bool) -> Result<reqwest::Client, Error> {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .map_err(std::convert::Into::into)
}

// so you can await? a result that might return Error
// https://www.lpalmieri.com/posts/error-handling-rust/#modelling-errors-as-enums
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // there is no lower level source for this error. Then None is appropriate
            Error::OhNo(_msg) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OhNo(msg) => writeln!(f, "{msg}"),
        }
    }
}

impl ResponseError for Error {
    fn status(&self) -> reqwest::StatusCode {
        match self {
            Error::OhNo(_msg) => reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::OhNo(e.to_string())
    }
}
