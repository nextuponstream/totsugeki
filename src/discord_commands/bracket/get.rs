//! Get bracket
use crate::discord_commands::bracket::Error;
use crate::TournamentServer;
use crate::{Bracket, Brackets};
use core::time::Duration;
use log::error;
use reqwest::StatusCode;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;

use super::get_client;

#[command]
#[description = "Get list of brackets"]
#[usage = "<OFFSET (use 0)>"]
// TODO use #[allowed_roles("TO")]
// https://github.com/serenity-rs/serenity/blob/current/examples/e12_global_data/src/main.rs
async fn get(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let offset = args.single::<i64>()?;

    let tournament_server = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<TournamentServer>()
            .expect("Expected TournamentServer in TypeMap.")
            .clone()
    };

    let brackets = match bot_get_bracket(
        tournament_server.get_connection_string(),
        offset,
        tournament_server.accept_invalid_certs,
    )
    .await
    {
        Ok(br) => br,
        Err(e) => {
            error!("{e}");
            return Err(e.into());
        }
    };
    let brackets = Brackets(brackets);
    if brackets.0.is_empty() {
        msg.reply(ctx, "None found").await?;
    } else {
        msg.reply(ctx, brackets.to_string()).await?;
    }
    Ok(())
}

/// Request to get brackets from discord bot
///
/// # Errors
/// Returns error when an error with the database occurs
pub async fn bot_get_bracket(
    tournament_server_addr: String,
    offset: i64,
    accept_invalid_certs: bool,
) -> Result<Vec<Bracket>, Error> {
    let client = get_client(accept_invalid_certs)?;
    let endpoint = format!("https://{tournament_server_addr}/bracket/{offset}");
    let res = client
        .get(endpoint)
        .timeout(Duration::from_secs(30))
        .send()
        .await?;
    if res.status() == StatusCode::OK {
        let brackets = res.json::<Vec<Bracket>>().await?;
        Ok(brackets)
    } else {
        Err(Error::OhNo(res.text().await?))
    }
}
