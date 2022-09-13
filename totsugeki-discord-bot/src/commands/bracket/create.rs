//! Create bracket

use crate::get_client;
use crate::Api;
use chrono::{NaiveDateTime, Utc};
use chrono_tz::Tz;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use totsugeki::bracket::http_responses::POST;
use totsugeki_api::Service;
use totsugeki_api_request::bracket::create as create_bracket;
use tracing::{info, span, warn, Level};

#[command]
#[description = "Create a new bracket. Respect double quotes."]
#[usage = "\"<NAME>\" YYYY-MM-DD:HH:MM TZ \"<FORMAT>\" \"<SEEDING METHOD>\" <AUTOMATIC BRACKET VALIDATION: true|false>"]
#[example = "\"mbtl weekly #1\" 2022-12-21:20:00 CET \"single-elimination\" \"strict\" true"]
#[allowed_roles("TO")]
// https://github.com/serenity-rs/serenity/blob/current/examples/e12_global_data/src/main.rs
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
        let format = args.single_quoted::<String>()?;
        let seeding_method = args.single_quoted::<String>()?;
        let automatic_match_validation = args.parse::<bool>()?;

        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };

        let organiser_id = msg.guild_id.expect("guild id");
        let organiser_id = organiser_id.to_string();
        let organiser_name = msg.guild(&ctx).expect("guild").name;
        let discussion_channel_id = msg.channel_id;
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
        let start_time= format!("{start_time:?}");

        let body = POST {
            bracket_name: bracket_name.clone(),
            organiser_name,
            organiser_internal_id: organiser_id,
            channel_internal_id: discussion_channel_id.to_string(),
            service_type_id: Service::Discord.to_string(),
            format,
            seeding_method,
            start_time,
            automatic_match_validation
        };
        create_bracket(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            api.get_authorization_header().as_str(),
            body,
        )
        .await?;
        info!("Bracket created");
        msg.reply(ctx, bracket_name).await?;
        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
