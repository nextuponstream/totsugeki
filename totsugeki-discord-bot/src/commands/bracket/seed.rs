//! Seed a bracket

use crate::{get_client, Api};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki::seeding::POST;
use totsugeki_api::Service;
use totsugeki_api_request::bracket::seed as seed_request;
use tracing::{error, span, Level};

#[command]
#[description = "Seed bracket by providing an ordered list of player IDs"]
#[usage = "<PLAYER IDS...>"]
#[allowed_roles("TO")]
async fn seed(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Seed bracket");
    span.in_scope(|| async {
        // NOTE: for now player ids are collect which means it does not have to
        // be in quotes
        let players = args.iter::<String>().collect::<Result<Vec<_>, _>>()?;
        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };
        let internal_channel_id = msg.channel_id.to_string();

        match seed_request(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            api.get_authorization_header().as_str(),
            POST {
                internal_channel_id,
                service: Service::Discord.to_string(),
                players,
            },
        )
        .await
        {
            Ok(id) => {
                // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
                msg.reply(ctx, format!("Bracket {id} seeded")).await?;
                Ok::<CommandResult, CommandError>(Ok(()))
            }
            Err(e) => {
                error!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                Err(e.into())
            }
        }
    })
    .await?
}
