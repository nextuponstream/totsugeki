//! Seed bracket command
//! Join bracket

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::format::Format;
use totsugeki_core::matches::result::MatchFormat;
use totsugeki_core::single_elimination_bracket::SingleEliminationBracket;
use totsugeki_core::validation::AutomaticMatchValidationMode;
use totsugeki_core::ID;
use tracing::{info, span, warn, Level};

#[command]
#[description = "Seed bracket by providing an ordered list of player IDs"]
#[usage = "<PLAYER IDS...>"]
#[allowed_roles("TO")]
async fn seed(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let span = span!(Level::INFO, "Seed bracket command");
    span.in_scope(|| async {
        let players = args.iter::<ID>().collect::<Result<Vec<_>, _>>()?;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (format, users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();

        let seeding = match Seeding::new(players.clone()) {
            Ok(r) => r,
            Err(e) => {
                warn!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };
        let data = match format {
            Format::SingleEliminationBracket => {
                let seb = SingleEliminationBracket::create(
                    seeding,
                    AutomaticMatchValidationMode::Flexible, // FIXME should be user provided
                    MatchFormat::ft3(),                     // FIXME should be user provided
                    None,                                   // FIXME should be user provided
                );
                *bracket_data = (
                    format,
                    users.clone(),
                    seb.clone(),
                    double_elimination_bracket.clone(),
                );
                Data {
                    users,
                    format,
                    single_elimination_bracket: Some(seb),
                    double_elimination_bracket: Some(double_elimination_bracket),
                }
            }
            Format::DoubleEliminationBracket => {
                let deb = DoubleEliminationBracket::create(
                    seeding,
                    AutomaticMatchValidationMode::Flexible, // FIXME should be user provided
                    MatchFormat::ft3(),                     // FIXME should be user provided
                    None,                                   // FIXME should be user provided
                );
                *bracket_data = (
                    format,
                    users.clone(),
                    single_elimination_bracket.clone(),
                    deb.clone(),
                );
                Data {
                    users,
                    format,
                    single_elimination_bracket: Some(single_elimination_bracket),
                    double_elimination_bracket: Some(deb),
                }
            }
        };

        let mut new_seeding_message = String::new();
        for p in players {
            new_seeding_message = format!("{new_seeding_message}\n- {p}");
        }

        let j = serde_json::to_string(&data).expect("bracket");

        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access
        let l: u64 = u64::try_from(j.len())?;
        f.set_len(l)?; // very important: if output has less chars than previous, output is padded
        f.write_all(j.as_bytes())?;

        info!("Seeding updated");
        msg.reply(ctx, format!("Seeding updated: {new_seeding_message}"))
            .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
