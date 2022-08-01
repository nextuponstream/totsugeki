//! Create bracket

use crate::get_client;
use crate::{Api, DiscordChannel};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki::bracket::RequestParameters;
use totsugeki_api_request::bracket::create as create_bracket;

#[command]
#[description = "Create a new bracket"]
#[usage = "<BRACKET NAME>"]
#[allowed_roles("TO")]
// https://github.com/serenity-rs/serenity/blob/current/examples/e12_global_data/src/main.rs
async fn create(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let bracket_name = args.single::<String>()?;

    let tournament_server = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<Api>()
            .expect("Expected TournamentServer in TypeMap.")
            .clone()
    };

    let organiser_id = msg.guild_id.expect("guild id");
    let organiser_id = organiser_id.to_string();
    let organiser_name = msg.guild(&ctx).expect("guild").name;
    let discussion_channel_id = msg.channel_id;
    let discord_channel = DiscordChannel::new(None, discussion_channel_id);
    let seeding_method = "strict"; // TODO request from args

    let parameters = RequestParameters {
        bracket_name: bracket_name.as_str(),
        bracket_format: "single-elimination",
        organiser_name: organiser_name.as_str(),
        organiser_id: organiser_id.as_str(),
        discussion_channel: discord_channel,
        seeding_method,
    };

    create_bracket(
        get_client(tournament_server.accept_invalid_certs)?,
        tournament_server.get_connection_string().as_str(),
        tournament_server.get_authorization_header().as_str(),
        parameters,
    )
    .await?;
    msg.reply(ctx, bracket_name).await?;
    Ok(())
}
