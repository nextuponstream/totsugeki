//! Quit before bracket starts

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use tracing::{info, span, warn, Level};

#[command]
#[description = "Quit bracket"]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Quit bracket");
    span.in_scope(|| async {
        let user_id = msg.author.id;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (mut bracket, users) = bracket_data.clone();

        let Some(player) = users.get(&user_id) else {
            warn!("Unregistered user");
            msg.reply(ctx, "You are not registered").await?;
            return Ok::<CommandResult, CommandError>(Ok(()));
        };

        bracket = match bracket.clone().remove_participant(player.get_id()) {
            Ok(b) => b,
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
            .truncate(false)
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access
        let l: u64 = u64::try_from(j.len())?;
        f.set_len(l)?; // very important: if output has less chars than previous, output is padded
        f.write_all(j.as_bytes())?;

        info!("{player} removed from bracket");
        msg.reply(ctx, format!("You ({player}) are removed from bracket"))
            .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
