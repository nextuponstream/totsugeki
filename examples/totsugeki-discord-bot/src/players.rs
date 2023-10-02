//! list players in bracket

use crate::Data;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use tracing::{span, Level};

#[command]
#[description = "List players in bracket"]
async fn players(ctx: &Context, msg: &Message) -> CommandResult {
    let span = span!(Level::INFO, "List players in bracket command");
    span.in_scope(|| async {
        let data = ctx.data.read().await;
        let bracket_data = data.get::<Data>().expect("data").clone();
        let bracket_data = bracket_data.read().await;
        let (bracket, _users) = bracket_data.clone();

        let players = bracket.clone().get_participants().get_players_list();
        let mut message = String::default();
        for p in players {
            message = format!("{message}\n{p}");
        }

        msg.reply(ctx, message).await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
