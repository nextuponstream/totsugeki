//! set of function to use
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
// reason, refactoring out of formatting string currently does not work
#![allow(clippy::uninlined_format_args)]
#![warn(clippy::unwrap_used)]
#![doc = include_str!("../README.md")]

use async_lock::RwLock;
use serde::{Deserialize, Serialize};
use serenity::{model::id::UserId, prelude::TypeMapKey};
use std::{collections::HashMap, sync::Arc};
use totsugeki::{bracket::Bracket, player::Player};

pub mod close;
pub mod create;
pub mod disqualify;
pub mod help;
pub mod join;
pub mod next_match;
pub mod ping;
pub mod players;
pub mod report;
pub mod start;
pub mod validate;
// mod find;
pub mod forfeit;
// mod get;
pub mod quit;
pub mod remove;
pub mod seed;

/// Configuration of saved file
pub struct Config;

impl TypeMapKey for Config {
    type Value = Arc<String>;
}

#[derive(Serialize, Deserialize, Debug)]
/// In-memory data to avoid reading save file
pub struct Data {
    /// Discord users
    pub users: HashMap<UserId, Player>,
    /// Hosted bracket
    pub bracket: Bracket,
}

impl TypeMapKey for Data {
    type Value = Arc<RwLock<(Bracket, HashMap<UserId, Player>)>>;
}
