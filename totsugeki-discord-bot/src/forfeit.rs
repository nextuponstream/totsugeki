//! Forfeit while bracket is running command

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use totsugeki::opponent::Opponent;
use tracing::{info, span, warn, Level};

#[command]
#[description = "Forfeit in running bracket"]
async fn forfeit(ctx: &Context, msg: &Message) -> CommandResult {
    let span = span!(Level::INFO, "Forfeit bracket");
    span.in_scope(|| async {
        let user_id = msg.author.id;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (mut bracket, users) = bracket_data.clone();

        let player = if let Some(p) = users.get(&user_id) {
            p.clone()
        } else {
            warn!("Unregistered user");
            msg.reply(ctx, "You are not registered").await?;
            return Ok::<CommandResult, CommandError>(Ok(()));
        };

        let mut new_matches_message = "".to_string();
        match bracket.clone().disqualify_participant(player.get_id()) {
            Ok((b, new_matches)) => {
                bracket = b;
                for m in new_matches {
                    let player1 = match m.get_players()[0].clone() {
                        Opponent::Player(p) => p,
                        Opponent::Unknown => panic!("cannot parse opponent"),
                    };
                    let player2 = match m.get_players()[1].clone() {
                        Opponent::Player(p) => p,
                        Opponent::Unknown => panic!("cannot parse opponent"),
                    };
                    new_matches_message = format!(
                        "{}\n{} VS {}\n- {}: {}\n- {}: {}",
                        new_matches_message,
                        player1.get_name(),
                        player2.get_name(),
                        player1.get_name(),
                        player1.get_id(),
                        player2.get_name(),
                        player2.get_id(),
                    );
                }
            }
            Err(e) => {
                warn!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };
        *bracket_data = (bracket.clone(), users.clone());

        let d = Data {
            bracket: bracket.clone(),
            users: users.clone(),
        };
        let j = serde_json::to_string(&d).expect("bracket");

        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access
        let l: u64 = u64::try_from(j.len())?;
        f.set_len(l)?; // very important: if output has less chars than previous, output is padded
        f.write_all(j.as_bytes())?;

        info!("{player} forfeited");
        msg.reply(
            ctx,
            format!("You have declared forfeit as {player}.{new_matches_message}"),
        )
        .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
