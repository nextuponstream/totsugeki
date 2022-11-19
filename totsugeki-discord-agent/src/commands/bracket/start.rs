//! Start bracket command

use crate::{get_client, Api, DiscordChannel};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki::opponent::Opponent;
use totsugeki_api_request::start::post;
use tracing::{span, warn, Level};

#[command]
#[description = "Start bracket. Allows people to start reporting match results."]
#[allowed_roles("TO")]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Start bracket");
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

        let response = match post(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            api.get_authorization_header().as_str(),
            discord_channel,
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                msg.reply(ctx, format!("{e}")).await?;
                warn!("User could not start bracket: {e}");
                return Err(e.into());
            }
        };

        let mut new_matches_message = String::new();
        for m in response.matches {
            let player1 = m.players[0].parse::<Opponent>().expect("opponent");
            let player1 = match player1 {
                Opponent::Player(p) => p,
                Opponent::Unknown => panic!("cannot parse opponent"),
            };
            let player2 = m.players[1].parse::<Opponent>().expect("opponent");
            let player2 = match player2 {
                Opponent::Player(p) => p,
                Opponent::Unknown => panic!("cannot parse opponent"),
            };
            new_matches_message = format!(
                "{}\n{} VS {}\n- {}: {}\n- {}: {}",
                new_matches_message,
                player1.get_name(),
                player2.get_name(),
                player1.get_name(),
                player1.get_id(),
                player2.get_name(),
                player2.get_id(),
            );
        }
        msg.reply(
            ctx,
            format!(
                "Bracket \"{}\" started.\n{new_matches_message}",
                response.bracket_id,
            ),
        )
        .await?;
        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
