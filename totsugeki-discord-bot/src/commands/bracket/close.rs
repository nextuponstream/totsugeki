//! Close bracket to prevent new participants from entering

use crate::{get_client, Api, DiscordChannel};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki_api_request::close::post;
use tracing::{span, warn, Level};

#[command]
#[description = "Close bracket. Prevent new participants from joining. May be used before seeding bracket to prevent surprise participations when seeding."]
#[allowed_roles("TO")]
async fn close(ctx: &Context, msg: &Message) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Close bracket");
    span.in_scope(|| async {
        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };
        let discussion_channel_id = msg.channel_id;
        let discord_channel = DiscordChannel::new(None, discussion_channel_id);

        let bracket_id = match post(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            api.get_authorization_header().as_str(),
            discord_channel,
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                msg.reply(ctx, format!("{e}")).await?;
                warn!("User could not close bracket: {e}");
                return Err(e.into());
            }
        };
        msg.reply(ctx, format!("Bracket \"{bracket_id}\" closed"))
            .await?;
        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
