use serenity::framework::standard::Args;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::connections::{on_play, on_leave};

use super::connections::on_stop;

#[command]
#[aliases("p")]
#[min_args(1)]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    log::info!("Play command");
    on_play(ctx, msg, args).await.unwrap();
    Ok(())
}

#[command]
#[aliases("s")]
#[only_in(guilds)]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Pause command");
    on_stop(ctx, msg).await.unwrap();
    Ok(())
}

#[command]
#[aliases("l")]
#[only_in(guilds)]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Leave command");
    on_stop(ctx, msg).await.unwrap();
    on_leave(ctx, msg).await.unwrap();
    Ok(())
}