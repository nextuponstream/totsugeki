//! Forfeit in bracket

use crate::{get_client, Api, DiscordChannel};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki_api_request::forfeit::post as forfeit_request;
use tracing::warn;
use tracing::{span, Level};

#[command]
#[description = "Forfeit"]
async fn forfeit(ctx: &Context, msg: &Message) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Forfeit bracket");
    span.in_scope(|| async {
        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };

        let user_id = msg.author.id;
        let discussion_channel_id = msg.channel_id;
        let discord_channel = DiscordChannel::new(None, discussion_channel_id);

        match forfeit_request(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            api.get_authorization_header().as_str(),
            discord_channel,
            user_id.to_string().as_str(),
        )
        .await
        {
            Ok(bracket_id) => {
                msg.reply(ctx, format!("You forfeited in bracket {bracket_id}"))
                    .await?;
                // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
                Ok::<CommandResult, CommandError>(Ok(()))
            }
            Err(e) => {
                msg.reply(ctx, format!("{e}")).await?;
                warn!("User could not forfeit in bracket: {e}");

                Err(e.into())
            }
        }
    })
    .await?
}
