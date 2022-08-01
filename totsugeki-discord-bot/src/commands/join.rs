//! join bracket as a player

use crate::{get_client, Api, DiscordChannel};
use reqwest::StatusCode;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use totsugeki_api_request::{join::post, RequestError};

#[command]
#[description = "Join active bracket in discussion channel"]
async fn join(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let user_id = msg.author.id;
    let user_name = msg.author.name.clone();
    let discussion_channel_id = msg.channel_id;
    let discord_channel = DiscordChannel::new(None, discussion_channel_id);
    let tournament_server = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<Api>()
            .expect("Expected TournamentServer in TypeMap.")
            .clone()
    };
    match post(
        get_client(tournament_server.accept_invalid_certs)?,
        tournament_server.get_connection_string().as_str(),
        tournament_server.get_authorization_header().as_str(),
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
            Ok(())
        }
        Err(e) => match e {
            RequestError::Request(re, e_msg) => {
                msg.reply(ctx, e_msg.as_str()).await?;
                if let Some(status) = re.status() {
                    if status == StatusCode::BAD_REQUEST {
                        log::warn!("{e_msg}");
                        return Ok(());
                    }
                }
                log::error!("{e_msg}");
                Err(Box::new(RequestError::Request(re, e_msg)))
            }
            RequestError::BracketParsingError(e) => {
                msg.reply(ctx, format!("{e}")).await?;
                log::warn!("User could not request bracket: {e}");
                Err(Box::new(RequestError::BracketParsingError(e)))
            }
        },
    }
}
