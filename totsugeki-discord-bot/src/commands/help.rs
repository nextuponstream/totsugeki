//! help command

use std::collections::HashSet;

use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::help;
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions,
};
use serenity::model::prelude::{Message, UserId};

struct Handler;

impl EventHandler for Handler {}

#[help]
#[lacking_permissions = "Hide"]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _msg = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
