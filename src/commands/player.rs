use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message) -> CommandResult{
    join(ctx, msg).await.unwrap();
    log::info!("Start playing music");
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult{
    log::info!("Pause playing music");
    Ok(())
}

pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guid = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guid.id;

    let channel_id = guid
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            log::warn!("User: {}; Is not in the voice channel", msg.author.name);
            msg.channel_id.say(&ctx.http, format!("{} please enter the voice channel", msg.author.name)).await.unwrap();
            return Ok(());
        }
    };
    
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation!!!").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    log::info!("Joined channel: {}", connect_to);
    Ok(())
}

pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _has_handler = manager.get(guild_id).is_some();

    log::info!("Left channel: {}",  guild_id);
    Ok(())
}