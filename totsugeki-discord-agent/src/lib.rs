//! discord commands

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

pub mod commands;

use poem::error::ResponseError;
use serenity::{model::id::ChannelId, prelude::TypeMapKey};
use std::sync::Arc;
use totsugeki::{DiscussionChannel, DiscussionChannelId};
use totsugeki_api::Service;

/// Tournament server
pub struct Api {
    /// url of Api
    addr: String,
    /// Api port (for local for local development)
    port: Option<String>,
    /// Parameter to accept self-signed certificate (for local development)
    accept_invalid_certs: bool,
    /// Authorization header to use api as registered user
    authorization_header: String,
}

impl TypeMapKey for Api {
    // While you will be using RwLock or Mutex most of the time you want to modify data,
    // sometimes it's not required; like for example, with static data, or if you are using other
    // kinds of atomic operators.
    //
    // Arc should stay, to allow for the data lock to be closed early.
    type Value = Arc<Api>;
}

impl Api {
    /// Create a new tournament server
    #[must_use]
    pub fn new(
        addr: String,
        port: Option<String>,
        accept_invalid_certs: bool,
        authorization_header: String,
    ) -> Self {
        Api {
            addr,
            port,
            accept_invalid_certs,
            authorization_header,
        }
    }

    /// Get connection string for tournament server
    #[must_use]
    pub fn get_connection_string(&self) -> String {
        if let Some(p) = self.port.clone() {
            format!("{}:{p}", self.addr)
        } else {
            self.addr.clone()
        }
    }

    /// Get authorization header with API token
    #[must_use]
    pub fn get_authorization_header(&self) -> String {
        self.authorization_header.clone()
    }
}

/// Discord discussion channel
#[derive(Debug, Clone)]
pub struct DiscordChannel {
    /// Api identifier of this discord channel
    channel_id: Option<DiscussionChannelId>,
    /// Discord identifier of this discussion channel
    internal_id: ChannelId,
}

impl DiscussionChannel for DiscordChannel {
    type InternalId = ChannelId;

    fn get_channel_id(&self) -> Option<DiscussionChannelId> {
        self.channel_id
    }

    fn get_internal_id(&self) -> Self::InternalId {
        self.internal_id
    }

    fn get_service_type(&self) -> String {
        Service::Discord.to_string()
    }
}

impl DiscordChannel {
    /// Create new discord channel
    #[must_use]
    pub fn new(channel_id: Option<DiscussionChannelId>, internal_id: ChannelId) -> Self {
        Self {
            channel_id,
            internal_id,
        }
    }
}

/// Get client that `accept_invalid_certs` for testing
fn get_client(accept_invalid_certs: bool) -> Result<reqwest::Client, Error> {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .map_err(std::convert::Into::into)
}
/// Errors while bot issues command
#[derive(Debug)]
pub enum Error {
    /// General errors
    ApiRequest(reqwest::Error),
}

// so you can await? a result that might return Error
// https://www.lpalmieri.com/posts/error-handling-rust/#modelling-errors-as-enums
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // there is no lower level source for this error. Then None is appropriate
            Error::ApiRequest(_msg) => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ApiRequest(e) => e.fmt(f),
        }
    }
}

impl ResponseError for Error {
    fn status(&self) -> reqwest::StatusCode {
        match self {
            Error::ApiRequest(_e) => reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::ApiRequest(e)
    }
}
