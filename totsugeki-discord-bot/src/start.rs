//! Join bracket

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use tracing::{info, span, Level};

#[command]
#[description = "Start bracket"]
#[allowed_roles("TO")]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    let span = span!(Level::INFO, "Start bracket command");
    span.in_scope(|| async {
        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (mut bracket, users) = bracket_data.clone();

        bracket = bracket.clone().start();
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
        f.set_len(l)?;
        f.write_all(j.as_bytes())?;

        info!("{bracket} started");
        msg.reply(ctx, format!("{bracket} started")).await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
