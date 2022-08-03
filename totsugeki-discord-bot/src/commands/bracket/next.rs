//! Next match to play for player

use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use totsugeki_api_request::next_match::next_match;

use crate::{get_client, Api, DiscordChannel};

#[command]
#[description = "Display next match to play"]
async fn next(ctx: &Context, msg: &Message) -> CommandResult {
    let tournament_server = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<Api>()
            .expect("Expected TournamentServer in TypeMap.")
            .clone()
    };

    let player_id = msg.author.id.to_string();
    let discussion_channel_id = msg.channel_id;
    let discord_channel = DiscordChannel::new(None, discussion_channel_id);
    let match_ = match next_match(
        get_client(tournament_server.accept_invalid_certs)?,
        tournament_server.get_connection_string().as_str(),
        player_id.as_str(),
        discord_channel,
    )
    .await
    {
        Ok(next_match) => next_match,
        Err(e) => {
            log::error!("{e}");
            return Err(e.into());
        }
    };

    // Something is seriously wrong if this happens
    if let totsugeki::matches::Opponent::Bye = match_.opponent {
        log::error!("Player has received a bye opponent. {match_}");
    };

    msg.reply(ctx, match_.to_string()).await?;

    Ok(())
}
