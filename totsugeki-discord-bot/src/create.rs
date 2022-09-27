//! Command to create a bracket

use chrono::{NaiveDateTime, Utc};
use chrono_tz::Tz;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use totsugeki::{bracket::Bracket, format::Format, seeding::Method};
use tracing::{info, span, warn, Level};
// use async_fs::File;
use crate::Config;
use std::{io::prelude::*, path::Path};
// use async_fs::File;
use fs4::FileExt;

#[command]
#[description = "Create a new bracket. Respect double quotes."]
#[usage = "\"<NAME>\" YYYY-MM-DD:HH:MM TZ \"<FORMAT>\" \"<SEEDING METHOD>\" <AUTOMATIC BRACKET VALIDATION: true|false>"]
#[example = "\"mbtl weekly #1\" 2022-12-21:20:00 CET \"single-elimination\" \"strict\" true"]
#[allowed_roles("TO")]
async fn create(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // TODO add description example values
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Create bracket command");
    span.in_scope(|| async {
        let bracket_name = args.single_quoted::<String>()?;
        let start_time = args.parse::<String>()?;
        args.advance();
        let tz = args.parse::<String>()?;
        args.advance();
        let format = args.single_quoted::<String>()?.parse::<Format>()?;
        let seeding_method = args.single_quoted::<String>()?.parse::<Method>()?;
        let automatic_match_validation = args.parse::<bool>()?;

        let start_time = match NaiveDateTime::parse_from_str(start_time.as_str(), "%Y-%m-%d:%H:%M")
        {
            Ok(s) => s,
            Err(e) => {
                warn!("User did not provide a correct date: {}, error: {}", start_time, e);
                msg.reply(
                    ctx,
                    "Error while parsing date, please use YYYY-MM-DD:HH:MM TZ",
                )
                .await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };
        let tz = match tz.parse::<Tz>() {
            Ok(tz) => tz,
            Err(e) => {
                warn!("User did not provide a correct timezone: {}, error: {}", tz, e);
                msg.reply(
                    ctx,
                    "Error while parsing timezone, please use YYYY-MM-DD:HH:MM TZ",
                )
                .await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };
        let start_time = match start_time.and_local_timezone(tz) {
            chrono::LocalResult::None => {
                warn!("User did not provide time: {}", tz);
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
            chrono::LocalResult::Single(st) => st,
            chrono::LocalResult::Ambiguous(dt1, dt2) => {
                warn!("User provided ambiguous time: {dt1}, {dt2}");
                msg.reply(
                    ctx,
                    format!("Using that time produced an ambiguous result ({} and {}). Please try another date.",dt1, dt2)
                )
                .await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            },
        };
        let start_time = start_time.with_timezone(&Utc);
      
        let bracket = Bracket::new(&bracket_name, format, seeding_method, start_time, automatic_match_validation);
        let data = ctx.data.read().await;
        let filename = data.get::<Config>().expect("filename").clone();
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(Path::new(filename.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access
        let j = serde_json::to_string(&bracket).expect("bracket");
        f.write_all(j.as_bytes())?;

        info!("Bracket created");
        msg.reply(ctx, bracket.to_string()).await?;

        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
