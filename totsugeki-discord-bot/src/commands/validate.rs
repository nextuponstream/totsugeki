//! validate match

use crate::{get_client, Api};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use totsugeki::matches::Id as MatchId;
use totsugeki_api_request::{validate::send, RequestError};
use tracing::error;
use tracing::{span, Level};

#[command]
#[description = "Validate reported results from player for a match"]
#[usage = "<MATCH ID>"]
#[allowed_roles("TO")]
async fn validate(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Validate match command");
    span.in_scope(|| async {
        let match_id = args.single::<MatchId>()?;
        let api = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<Api>()
                .expect("Expected Api in TypeMap.")
                .clone()
        };
        if let Err(e) = send(
            get_client(api.accept_invalid_certs)?,
            api.get_connection_string().as_str(),
            api.get_authorization_header().as_str(),
            match_id,
        )
        .await
        {
            match e {
                // NOTE: ref to avoid borrowing
                RequestError::Request(ref _re, ref e_msg) => {
                    msg.reply(ctx, e_msg).await?;
                    error!("{}", e_msg);
                }
                RequestError::BracketParsingError(ref e) => {
                    msg.reply(ctx, format!("{e}")).await?;
                    error!("User could not request bracket: {e}");
                }
                RequestError::MatchIdParsingError(ref e) => {
                    msg.reply(ctx, format!("{e}")).await?;
                    error!("User could not request bracket: {e}");
                }
                RequestError::NextMatch(ref e) => {
                    msg.reply(ctx, e.to_string()).await?;
                    error!("User could not request bracket: {e}");
                }
            };
            Err(e.into())
        } else {
            msg.reply(ctx, "Match validated. Bracket updated successfully.")
                .await?;
            // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
            Ok::<CommandResult, CommandError>(Ok(()))
        }
    })
    .await?
}
