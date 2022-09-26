//! Ping command for discord bot

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::{span, Level};

#[command]
#[description = "Ping the bot"]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Ping command");
    span.in_scope(|| async {
        msg.reply(ctx, "Pong!").await?;

        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
