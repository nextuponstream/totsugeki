//! Bracket management commands
mod close;
mod create;
mod find;
mod get;
mod players;
mod quit;
mod seed;
mod start;

use self::{
    close::CLOSE_COMMAND, create::CREATE_COMMAND, find::FIND_COMMAND, get::GET_COMMAND,
    players::PLAYERS_COMMAND, quit::QUIT_COMMAND, seed::SEED_COMMAND, start::START_COMMAND,
};
use serenity::framework::standard::macros::group;

#[group]
#[allow(missing_docs)]
#[prefix("bracket")]
#[commands(create, get, find, start, seed, players, close, quit)]
#[summary = "Main available commands"]
#[description = "Subcommand for TO's to manage a bracket"]
pub struct Bracket;
