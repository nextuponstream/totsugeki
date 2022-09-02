//! Return players of bracket

use crate::{get_client, Api};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki::player::GET;
use totsugeki_api::Service;
use totsugeki_api_request::bracket::fetch_players;
use tracing::{error, span, Level};

#[command]
#[description = "Return list of players for active bracket"]
async fn players(ctx: &Context, msg: &Message) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "List players of active bracket");
    span.in_scope(|| async {
        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };
        let discussion_channel_id = msg.channel_id;
        let (bracket_id, players) = match fetch_players(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            GET {
                internal_discussion_channel_id: discussion_channel_id.to_string(),
                service: Service::Discord.to_string(),
            },
        )
        .await
        {
            Ok(response) => response,
            Err(e) => {
                error!("{e}");
                return Err(e.into());
            }
        };
        msg.reply(ctx, format!("{bracket_id}:\n{players}")).await?;
        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
