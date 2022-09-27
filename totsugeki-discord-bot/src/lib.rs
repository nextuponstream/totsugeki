//! set of function to use

use async_lock::RwLock;
use serde::{Deserialize, Serialize};
use serenity::{model::id::UserId, prelude::TypeMapKey};
use std::{collections::HashMap, sync::Arc};
use totsugeki::{bracket::Bracket, player::Player};

pub mod create;
pub mod help;
pub mod join;
pub mod ping;
pub mod report;
pub mod start;

pub struct Config;

impl TypeMapKey for Config {
    type Value = Arc<String>;
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub bracket: Bracket,
    pub users: HashMap<UserId, Player>,
}

impl TypeMapKey for Data {
    type Value = Arc<RwLock<(Bracket, HashMap<UserId, Player>)>>;
}
