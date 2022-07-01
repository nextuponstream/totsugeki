//! Create bracket

use super::get_client;
use crate::TournamentServer;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use tournament_server_request::bracket::create as create_bracket;

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
    create_bracket(
        get_client(tournament_server.accept_invalid_certs)?,
        tournament_server.get_connection_string().as_str(),
        bracket_name.to_string(),
    )
    .await?;
    msg.reply(ctx, bracket_name).await?;
    Ok(())
}
