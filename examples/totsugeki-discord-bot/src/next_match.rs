//! command for players to display their next match

use crate::Data;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use totsugeki_core::format::Format;
use totsugeki_core::next_opponent::NextOpponentInBracket;
use totsugeki_core::player::Player;
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
        let (format, users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();

        let player = match users.get(&user_id) {
            Some(p) => p.clone(),
            None => Player::new(name),
        };

        let (opponent, match_id) = match format {
            Format::SingleEliminationBracket => {
                match single_elimination_bracket.next_opponent_in_bracket(player.get_id()) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("{e}");
                        msg.reply(ctx, format!("{e}")).await?;
                        return Ok::<CommandResult, CommandError>(Ok(()));
                    }
                }
            }
            Format::DoubleEliminationBracket => {
                match double_elimination_bracket.next_opponent_in_bracket(player.get_id()) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("{e}");
                        msg.reply(ctx, format!("{e}")).await?;
                        return Ok::<CommandResult, CommandError>(Ok(()));
                    }
                }
            }
        };

        info!("{player} joined");
        msg.reply(
            ctx,
            format!("Your next opponent is {opponent} in match {match_id}"),
        )
        .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
