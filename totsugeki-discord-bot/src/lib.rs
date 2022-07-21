//! discord commands

#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

pub mod commands;

use serenity::{model::id::ChannelId, prelude::TypeMapKey};
use std::sync::Arc;
use totsugeki::{DiscussionChannel, DiscussionChannelId};
use totsugeki_api::InternalIdType;

/// Tournament server
pub struct TournamentServer {
    addr: String,
    port: Option<String>,
    accept_invalid_certs: bool,
    authorization_header: String,
}

impl TypeMapKey for TournamentServer {
    // While you will be using RwLock or Mutex most of the time you want to modify data,
    // sometimes it's not required; like for example, with static data, or if you are using other
    // kinds of atomic operators.
    //
    // Arc should stay, to allow for the data lock to be closed early.
    type Value = Arc<TournamentServer>;
}

impl TournamentServer {
    /// Create a new tournament server
    #[must_use]
    pub fn new(
        addr: String,
        port: Option<String>,
        accept_invalid_certs: bool,
        authorization_header: String,
    ) -> Self {
        TournamentServer {
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
    channel_id: Option<DiscussionChannelId>,
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
        InternalIdType::Discord.to_string()
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
