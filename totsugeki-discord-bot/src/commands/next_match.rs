//! Show next match in bracket

use log::{error, warn};
use reqwest::StatusCode;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use totsugeki_api_request::{next_match::next_match as next_match_request, RequestError};

use crate::{get_client, Api, DiscordChannel};

#[command]
#[description = "Show next match. Available in the same discussion channel of the active bracket."]
async fn next_match(ctx: &Context, msg: &Message) -> CommandResult {
    let tournament_server = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<Api>()
            .expect("Expected TournamentServer in TypeMap.")
            .clone()
    };
    let discussion_channel_id = msg.channel_id;
    let discord_channel = DiscordChannel::new(None, discussion_channel_id);
    let player_internal_id = msg.author.id;

    let next_match = match next_match_request(
        get_client(tournament_server.accept_invalid_certs)?,
        &tournament_server.get_connection_string(),
        &tournament_server.get_authorization_header(),
        &player_internal_id.to_string(),
        discord_channel,
    )
    .await
    {
        Ok(br) => br,
        Err(e) => {
            if let RequestError::Request(e, m) = e {
                // Someone may have thought they still had a match to play
                if let Some(s) = e.status() {
                    if s == StatusCode::NOT_FOUND {
                        warn!("{e}:{m}");
                        msg.reply(ctx, m).await?;
                        return Ok(());
                    }
                }

                error!("{e:?}:{m}"); // otherwise, it is error worthy
                msg.reply(
                    ctx,
                    format!("An error happened while searching for next match: {e}"),
                )
                .await?;
                return Err(e.into());
            }

            error!("{e:?}");
            msg.reply(
                ctx,
                format!("An error happened while searching for next match: {e}"),
            )
            .await?;
            return Err(e.into());
        }
    };

    msg.reply(ctx, next_match.to_string()).await?;
    Ok(())
}
