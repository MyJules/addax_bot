use serenity::framework::standard::Args;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::connections::{bot_play, bot_leave, bot_skip, bot_pause, bot_stop};

#[command]
#[aliases("p")]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    log::info!("Play command");
    bot_play(ctx, msg, args).await.unwrap();
    Ok(())
}

#[command]
#[aliases("s")]
#[only_in(guilds)]
pub async fn skip(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    log::info!("Skip command");
    bot_skip(ctx, msg).await.unwrap();
    Ok(())
}

#[command]
#[aliases("ps")]
#[only_in(guilds)]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Pause command");
    bot_pause(ctx, msg).await.unwrap();
    Ok(())
}

#[command]
#[aliases("l")]
#[only_in(guilds)]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Leave command");
    bot_stop(ctx, msg).await.unwrap();
    bot_leave(ctx, msg).await.unwrap();
    Ok(())
}