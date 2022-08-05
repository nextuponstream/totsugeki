//! Report match results

use log::error;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use totsugeki_api_request::report::result;

use crate::{get_client, Api, DiscordChannel};

#[command]
#[description = "Report result of your match. Available in the same discussion channel of the active bracket."]
#[usage = "<RESULT (2-0, 0-2, 1-2...)>"]
async fn report(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let reported_result = args.single::<String>()?;

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

    let match_id = match result(
        get_client(tournament_server.accept_invalid_certs)?,
        &tournament_server.get_connection_string(),
        &tournament_server.get_authorization_header(),
        &player_internal_id.to_string(),
        &reported_result,
        discord_channel,
    )
    .await
    {
        Ok(br) => br,
        Err(e) => {
            error!("{e}");
            msg.reply(ctx, "An error happened while updating match result.")
                .await?;
            return Err(e.into());
        }
    };

    msg.reply(ctx, format!("Result reported for match: {match_id}"))
        .await?;
    Ok(())
}
