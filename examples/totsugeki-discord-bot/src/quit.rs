//! Quit before bracket starts

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::format::Format;
use totsugeki::matches::result::MatchFormat;
use totsugeki::player::PlayerID;
use totsugeki::single_elimination_bracket::SingleEliminationBracket;
use totsugeki::validation::AutomaticMatchValidationMode;
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
        let (format, users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();

        let users_copy = users.clone();
        let Some(player) = users_copy.get(&user_id) else {
            warn!("Unregistered user");
            msg.reply(ctx, "You are not registered").await?;
            return Ok::<CommandResult, CommandError>(Ok(()));
        };

        let data = match format {
            Format::SingleEliminationBracket => {
                let seeding = single_elimination_bracket.get_seeding().get();
                let seeding = seeding
                    .into_iter()
                    .filter(|id| *id != player.get_id())
                    .collect::<Vec<PlayerID>>();
                let seb = SingleEliminationBracket::create(
                    Seeding::new(seeding).expect("seeding should not contain user"),
                    AutomaticMatchValidationMode::Flexible,
                    MatchFormat::ft3(), // FIXME should be user provided
                    None,               // FIXME should be user provided
                );
                *bracket_data = (
                    format,
                    users.clone(),
                    seb.clone(),
                    double_elimination_bracket,
                );
                Data {
                    users,
                    format,
                    single_elimination_bracket: Some(seb),
                    double_elimination_bracket: None,
                }
            }
            Format::DoubleEliminationBracket => {
                let seeding = double_elimination_bracket.get_seeding().get();
                let seeding = seeding
                    .into_iter()
                    .filter(|id| *id != player.get_id())
                    .collect::<Vec<PlayerID>>();
                let deb = DoubleEliminationBracket::create(
                    Seeding::new(seeding).expect("seeding should not contain user"),
                    AutomaticMatchValidationMode::Flexible,
                    MatchFormat::ft3(), // FIXME should be user provided
                    None,               // FIXME should be user provided
                );
                *bracket_data = (
                    format,
                    users.clone(),
                    single_elimination_bracket,
                    deb.clone(),
                );
                Data {
                    users,
                    format,
                    single_elimination_bracket: None,
                    double_elimination_bracket: Some(deb),
                }
            }
        };

        let j = serde_json::to_string(&data).expect("bracket");

        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access
        let l: u64 = u64::try_from(j.len())?;
        f.set_len(l)?; // very important: if output has fewer chars than previous, output is padded
        f.write_all(j.as_bytes())?;

        info!("{player} removed from bracket");
        msg.reply(ctx, format!("You ({player}) are removed from bracket"))
            .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
