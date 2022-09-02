//! Bracket management commands
pub mod create;
pub mod find;
pub mod get;
pub mod players;
pub mod seed;
pub mod start;

use self::create::CREATE_COMMAND;
use self::find::FIND_COMMAND;
use self::get::GET_COMMAND;
use self::players::PLAYERS_COMMAND;
use self::seed::SEED_COMMAND;
use self::start::START_COMMAND;
use serenity::framework::standard::macros::group;

#[group]
#[allow(missing_docs)]
#[prefix("bracket")]
#[commands(create, get, find, start, seed, players)]
#[summary = "Main available commands"]
#[description = "Subcommand for TO's to manage a bracket"]
pub struct Bracket;
