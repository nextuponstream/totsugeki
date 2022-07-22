//! join bracket as a player

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

#[command]
async fn join(_ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let _id = msg.author.id;
    todo!()
}
