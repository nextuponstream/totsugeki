//! Report match results

use crate::{get_client, Api, DiscordChannel};
use reqwest::StatusCode;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use totsugeki_api_request::{report::result, RequestError};
use tracing::{error, warn};
use tracing::{span, Level};

#[command]
#[description = "Report result of your match. Available in the same discussion channel of the active bracket."]
#[usage = "<RESULT (2-0, 0-2, 1-2...)>"]
async fn report(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Report match result command");
    span.in_scope(|| async {
        let reported_result = args.single::<String>()?;

        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };
        let discussion_channel_id = msg.channel_id;
        let discord_channel = DiscordChannel::new(None, discussion_channel_id);
        let player_internal_id = msg.author.id;

        let match_id = match result(
            get_client(api.accept_invalid_certs)?,
            &api.get_connection_string(),
            &api.get_authorization_header(),
            &player_internal_id.to_string(),
            &reported_result,
            discord_channel,
        )
        .await
        {
            Ok(br) => br,
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

                return Err(e.into());
            }
        };

        msg.reply(ctx, format!("Result reported for match: {match_id}"))
            .await?;
        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
