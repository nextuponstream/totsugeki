//! Validate match as tournament organiser

use crate::{Config, Data};
use std::{io::prelude::*, path::Path};
// use async_fs::File;
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use totsugeki::double_elimination_bracket::progression::ProgressionDEB;
use totsugeki::format::Format;
use totsugeki::matches::MatchID;
use totsugeki::single_elimination_bracket::progression::ProgressionSEB;
use totsugeki::{matches::Id as MatchId, opponent::Opponent};
use tracing::{info, span, warn, Level};

#[command]
#[description = "Validate match in bracket"]
#[allowed_roles("TO")]
async fn validate(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let span = span!(Level::INFO, "Validate match in bracket command");
    span.in_scope(|| async {
        let match_id = args.parse::<MatchID>()?;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (format, users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();

        let (data, new_matches) = match format {
            Format::SingleEliminationBracket => {
                let (seb, new_matches) = single_elimination_bracket.validate_match_result(match_id);
                (
                    Data {
                        users,
                        format,
                        single_elimination_bracket: Some(seb),
                        double_elimination_bracket: None,
                    },
                    new_matches,
                )
            }
            Format::DoubleEliminationBracket => {
                let (deb, new_matches) = double_elimination_bracket.validate_match_result(match_id);
                (
                    Data {
                        users,
                        format,
                        single_elimination_bracket: None,
                        double_elimination_bracket: Some(deb),
                    },
                    new_matches,
                )
            }
        };

        let mut new_matches_message = String::new();
        for m in new_matches {
            let player1 = match m.get_players()[0] {
                Opponent(Some(p)) => p,
                Opponent(None) => panic!("cannot parse opponent"),
            };
            let player2 = match m.get_players()[1] {
                Opponent(Some(p)) => p,
                Opponent(None) => panic!("cannot parse opponent"),
            };
            new_matches_message = format!("{}\n{} VS {}", new_matches_message, player1, player2);
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

        info!("Match validated in bracket");
        msg.reply(
            ctx,
            format!("You validated {match_id}.{new_matches_message}"),
        )
        .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
