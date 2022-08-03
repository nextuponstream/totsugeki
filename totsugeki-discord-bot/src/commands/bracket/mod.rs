//! Bracket management commands
pub mod create;
pub mod find;
pub mod get;
pub mod next;

use self::create::CREATE_COMMAND;
use self::find::FIND_COMMAND;
use self::get::GET_COMMAND;
use serenity::framework::standard::macros::group;

#[group]
#[allow(missing_docs)]
#[prefix("bracket")]
#[commands(create, get, find)]
#[summary = "Main available commands"]
#[description = "Subcommand for TO's to manage a bracket"]
pub struct Bracket;
