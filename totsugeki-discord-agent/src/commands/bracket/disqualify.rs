//! TO can disqualify player after bracket has started

use crate::Service;
use crate::{get_client, Api};
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki::remove::POST;
use totsugeki_api_request::disqualify::post;
use tracing::{error, span, Level};

#[command]
#[description = "Disqualify player from bracket"]
#[allowed_roles("TO")]
#[aliases("dq")]
async fn disqualify(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Disqualify player from bracket");
    span.in_scope(|| async {
        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };

        let player_id = args.single::<String>()?;

        let discussion_channel_id = msg.channel_id;
        let bracket_id = match post(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            api.get_authorization_header().as_str(),
            POST {
                internal_channel_id: discussion_channel_id.to_string(),
                player_id: player_id.clone(),
                service: Service::Discord.to_string(),
            },
        )
        .await
        {
            Ok(bracket_id) => bracket_id,
            Err(e) => {
                error!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                return Err(e.into());
            }
        };
        msg.reply(
            ctx,
            format!("Disqualified {player_id} from bracket {bracket_id}"),
        )
        .await?;
        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
