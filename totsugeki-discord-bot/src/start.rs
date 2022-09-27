//! start bracket command

use crate::{Config, Data};
use std::{io::prelude::*, path::Path};
// use async_fs::File;
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use tracing::{info, span, Level};

#[command]
#[description = "Start bracket. Allows people to start reporting match results."]
#[allowed_roles("TO")]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    let span = span!(Level::INFO, "Start bracket command");
    span.in_scope(|| async {
        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (bracket, users) = bracket_data.clone();
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access

        let bracket = bracket.clone().start();
        *bracket_data = (bracket.clone(), users);

        let j = serde_json::to_string(&bracket_data.clone()).expect("bracket");
        f.write_all(j.as_bytes())?;
        msg.reply(ctx, format!("{bracket} started")).await?;
        info!("{bracket} started");
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
