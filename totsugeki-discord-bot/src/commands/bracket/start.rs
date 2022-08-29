//! Start bracket command

use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::*;

#[command]
#[description = "Start bracket. Allows people to start reporting match results."]
#[allowed_roles("TO")]
async fn start(_ctx: &Context, _msg: &Message) -> CommandResult {
    todo!()
}
