//! list players in bracket

use crate::Data;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use totsugeki_core::format::Format;
use tracing::{span, Level};

#[command]
#[description = "List players in bracket"]
async fn players(ctx: &Context, msg: &Message) -> CommandResult {
    let span = span!(Level::INFO, "List players in bracket command");
    span.in_scope(|| async {
        let data = ctx.data.read().await;
        let bracket_data = data.get::<Data>().expect("data").clone();
        let bracket_data = bracket_data.read().await;
        let (format, _users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();

        let players_ids = match format {
            Format::SingleEliminationBracket => single_elimination_bracket.get_seeding().get(),
            Format::DoubleEliminationBracket => double_elimination_bracket.get_seeding().get(),
        };
        // TODO add Tournament struct with player names
        let mut message = String::default();
        for p in players_ids {
            message = format!("{message}\n{p}");
        }

        msg.reply(ctx, message).await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
