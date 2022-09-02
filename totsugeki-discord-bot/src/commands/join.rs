//! join bracket as a player

use crate::{get_client, Api, DiscordChannel};
use reqwest::StatusCode;
use serenity::{
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
    prelude::*,
};
use totsugeki_api_request::{join::post, RequestError};
use tracing::{error, warn};
use tracing::{span, Level};

#[command]
#[description = "Join active bracket in discussion channel"]
async fn join(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Join command");
    span.in_scope(|| async {
        let user_id = msg.author.id;
        let user_name = msg.author.name.clone();
        let discussion_channel_id = msg.channel_id;
        let discord_channel = DiscordChannel::new(None, discussion_channel_id);
        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };
        match post(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            api.get_authorization_header().as_str(),
            user_id.to_string().as_str(),
            user_name.as_str(),
            discord_channel,
        )
        .await
        {
            Ok(response) => {
                msg.reply(
                    ctx,
                    format!("You have joined bracket {}", response.bracket_id),
                )
                .await?;
                // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
                Ok::<CommandResult, CommandError>(Ok(()))
            }
            Err(e) => {
                match e {
                    RequestError::Request(ref re, ref e_msg) => {
                        msg.reply(ctx, e_msg.as_str()).await?;
                        if let Some(status) = re.status() {
                            match status {
                                StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN => {
                                    warn!("{e_msg}");
                                    return Ok::<CommandResult, CommandError>(Ok(()));
                                }
                                _ => {}
                            };
                        }
                        error!("{e_msg}");
                    }
                    RequestError::BracketParsingError(ref e) => {
                        msg.reply(ctx, format!("{e}")).await?;
                        warn!("User could not request bracket: {e}");
                    }
                    RequestError::MatchIdParsingError(ref e) => {
                        msg.reply(ctx, format!("{e}")).await?;
                        warn!("User could not request bracket: {e}");
                    }
                    RequestError::NextMatch(ref e) => {
                        msg.reply(ctx, format!("{e}")).await?;
                        error!("User could not request bracket: {e}");
                    }
                    RequestError::PlayerParsingError(ref e) => {
                        msg.reply(ctx, format!("{e}")).await?;
                        error!("User could not request bracket: {e}");
                    }
                };

                Err(e.into())
            }
        }
    })
    .await?
}
