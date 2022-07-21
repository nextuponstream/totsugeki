//! Get bracket
use crate::TournamentServer;
use log::error;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki::bracket::Brackets;
use totsugeki_api_request::bracket::fetch;

use super::get_client;

#[command]
#[description = "Get list of brackets"]
#[usage = "<OFFSET (use 0)>"]
async fn get(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let offset = args.single::<i64>()?;

    let tournament_server = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<TournamentServer>()
            .expect("Expected TournamentServer in TypeMap.")
            .clone()
    };

    let brackets = match fetch(
        get_client(tournament_server.accept_invalid_certs)?,
        tournament_server.get_connection_string().as_str(),
        None,
        offset,
    )
    .await
    {
        Ok(br) => br,
        Err(e) => {
            error!("{e}");
            return Err(e.into());
        }
    };
    let brackets = Brackets::new(brackets);
    if brackets.get_brackets().is_empty() {
        msg.reply(ctx, "None found").await?;
    } else {
        msg.reply(ctx, brackets.to_string()).await?;
    }
    Ok(())
}
