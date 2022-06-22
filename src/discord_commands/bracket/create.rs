//! Create bracket

use crate::discord_commands::bracket::Error;
use crate::{BracketPOST, TournamentServer};
use core::time::Duration;
use reqwest::StatusCode;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;

use super::get_client;

#[command]
#[description = "Create a new bracket"]
#[usage = "<BRACKET NAME>"]
// TODO use #[allowed_roles("TO")]
// https://github.com/serenity-rs/serenity/blob/current/examples/e12_global_data/src/main.rs
async fn create(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let bracket_name = args.single::<String>()?;

    let tournament_server = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<TournamentServer>()
            .expect("Expected TournamentServer in TypeMap.")
            .clone()
    };
    bot_create_bracket(
        tournament_server.get_connection_string(),
        bracket_name.to_string(),
        tournament_server.accept_invalid_certs,
    )
    .await?;
    msg.reply(ctx, bracket_name).await?;
    Ok(())
}

/// Request to create a bracket from discord bot
///
/// # Errors
pub async fn bot_create_bracket(
    tournament_server_addr: String,
    bracket_name: String,
    accept_invalid_certs: bool,
) -> Result<(), Error> {
    let endpoint = format!("https://{tournament_server_addr}/bracket");
    let body = BracketPOST::new(bracket_name.clone());
    let client = get_client(accept_invalid_certs)?;
    let res = client
        .post(endpoint)
        .timeout(Duration::from_secs(30))
        .json(&body)
        .send()
        .await?;
    if res.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(Error::OhNo(res.text().await?))
    }
}
