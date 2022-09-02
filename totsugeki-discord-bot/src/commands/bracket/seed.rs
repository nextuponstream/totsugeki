//! Seed a bracket

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::{span, Level};

#[command]
#[description = "Seed bracket by providing an ordered list of player IDs"]
#[usage = "<PLAYER IDS...>"]
#[allowed_roles("TO")]
async fn seed(_ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Seed bracket");
    span.in_scope(|| async {
        for _arg in args.iter::<String>() {
            todo!()
        }

        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
