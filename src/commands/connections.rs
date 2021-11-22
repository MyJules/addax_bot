use std::time::Duration;

use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::disconnect_handler::DisconnectHandler;

pub async fn connect(ctx: &Context, msg: &Message) -> CommandResult {
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
        .expect("Songbird Voice client placed in at initialization!!!").clone();
    let (handler_lock, conn_result) = manager.join(guild_id, connect_to).await;

    if let Ok(_) = conn_result{
        let mut handler = handler_lock.lock().await;

        handler.add_global_event(
            songbird::Event::Periodic(Duration::from_millis(3000), None),
            DisconnectHandler::new(manager, guid, ctx.clone(), connect_to),
        );
    }

    log::info!("Joined channel: {}", connect_to);
    Ok(())
}


pub async fn disconnect(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialization.").clone();

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.leave(guild_id).await {
            log::error!("Error disconnecting from voice channel: {:?}", e);
        }
        log::info!("Bot left channel");
    }else{
        log::warn!("Bot is not in the voice channel");
    }

    Ok(())
}