#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

pub mod discord_commands;
pub mod persistence;
pub mod routes;

use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;
use std::fmt::Display;
use std::sync::Arc;

// TODO yew web frontend

/// Tournament organiser
pub struct TO {}

impl TO {
    /// create a new tournament organiser
    #[must_use]
    pub fn default() -> Self {
        TO {}
    }
}

/// Tournament server
pub struct TournamentServer {
    addr: String,
    port: Option<String>,
    accept_invalid_certs: bool,
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
    pub fn new(addr: String, port: Option<String>, accept_invalid_certs: bool) -> Self {
        TournamentServer {
            addr,
            port,
            accept_invalid_certs,
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
}

#[derive(Serialize, Deserialize, Object)]
/// Body of bracket POST request
pub struct BracketPOST {
    /// name of the bracket
    pub bracket_name: String,
}

impl BracketPOST {
    fn new(bracket_name: String) -> Self {
        BracketPOST { bracket_name }
    }
}

/// Bracket for a tournament
#[derive(Debug, Serialize, Deserialize, Object, Clone)]
pub struct Bracket {
    id: i64, // TODO change to UUID
    bracket_name: String,
}

impl Display for Bracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{{ id: {}, bracket_name \"{} \"}}",
            self.id, self.bracket_name
        )
    }
}

struct Brackets(Vec<Bracket>);

impl Display for Brackets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.0.clone() {
            b.fmt(f)?;
        }
        Ok(())
    }
}

impl Bracket {
    /// Create new bracket
    #[must_use]
    pub fn new(id: i64, bracket_name: String) -> Self {
        // TODO add check where registration_start_time < beginning_start_time
        Bracket { id, bracket_name }
    }

    /// Get name of bracket
    #[must_use]
    pub fn get_bracket_name(self) -> String {
        self.bracket_name
    }
}

// Player
//struct Player {
//    id: i32, // TODO use UUID
//}

// Bracket is run by a `TO` where `players` participate
//struct Bracket {
//    to: TO,
//    players: Vec<Player>,
//    format: TournamentFormat,
//    registration_start_time: String, // TODO use utc date
//    beginning_start_time: String,    // TODO use utc date
//}

//enum BracketError {
//    Invalid,
//}

// Tournament format
//enum TournamentFormat {
//    DoubleElimination,
//}
