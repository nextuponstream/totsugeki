//! Get bracket
use crate::{get_client, Api};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki::bracket::Brackets;
use totsugeki_api_request::bracket::fetch;
use tracing::error;
use tracing::{span, Level};

#[command]
#[description = "Get list of brackets"]
#[usage = "<OFFSET (use 0)>"]
async fn get(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "List brackets command");
    span.in_scope(|| async {
        let offset = args.single::<i64>()?;

        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };

        let brackets = match fetch(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            None,
            offset,
        )
        .await
        {
            Ok(br) => br,
            Err(e) => {
                error!("{e}");
                return Err(e.into());
            }
        };
        let brackets = Brackets::new(brackets);
        if brackets.get_brackets().is_empty() {
            msg.reply(ctx, "None found").await?;
        } else {
            msg.reply(ctx, brackets.to_string()).await?;
        }
        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
