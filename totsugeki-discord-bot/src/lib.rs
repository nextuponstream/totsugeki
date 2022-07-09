//! discord commands

#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

pub mod commands;

use serenity::prelude::TypeMapKey;
use std::sync::Arc;

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
