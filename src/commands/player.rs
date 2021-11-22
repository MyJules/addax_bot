use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::connections::{connect, disconnect};

#[command]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    connect(ctx, msg).await.unwrap();
    log::info!("Play command");
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Pause command");
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    disconnect(ctx, msg).await.unwrap();
    log::info!("Leave command");
    Ok(())
}