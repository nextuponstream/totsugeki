//! command for players to display their next match

use crate::Data;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use totsugeki::player::Player;
use tracing::{info, span, warn, Level};

#[command]
#[description = "Next match in bracket"]
async fn next_match(ctx: &Context, msg: &Message) -> CommandResult {
    let span = span!(Level::INFO, "Next match in bracket command");
    span.in_scope(|| async {
        let name = msg.author.name.clone();
        let user_id = msg.author.id;

        let data = ctx.data.read().await;
        let bracket_data = data.get::<Data>().expect("data").clone();
        let bracket_data = bracket_data.read().await;
        let (bracket, users) = bracket_data.clone();

        let player = match users.get(&user_id) {
            Some(p) => p.clone(),
            None => Player::new(name),
        };

        let (opponent, m, _player_name) = match bracket.clone().next_opponent(player.get_id()) {
            Ok(r) => r,
            Err(e) => {
                warn!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };

        info!("{player} joined");
        msg.reply(
            ctx,
            format!("Your next opponent is {opponent} in match {m}"),
        )
        .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
