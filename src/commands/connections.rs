use std::sync::Arc;

use serenity::framework::standard::Args;
use songbird::{create_player, TrackEvent};
use songbird::driver::Bitrate;
use songbird::input::{Input, Restartable};
use std::time::Duration;

use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::disconnect_if_no_user_left::DisconnectIfNoUsers;
use crate::commands::disconnect_if_queue_empty::DisconnectIfPlayerQueueEmpty;

pub async fn on_play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    let (handler_lock, is_connected) = manager.join(guild_id, connect_to).await;
    
    if let Ok(_) = is_connected {
        let mut handler = handler_lock.lock().await;

        //parse args
        let args = match args.len() {
            0 => {
                log::warn!("Zero arguments in play command");
                Err("Please give a url or name of the song")
            },
            1 => {
                log::info!("Play command: {}", args.message());
                args.single::<String>().map_err(|_| "Please give a url or name of the song")
            },
            _ => Ok(args.iter::<String>().fold(String::new(), |mut a, b| {
                a.push(' ');
                a.push_str(&b.unwrap());
                a
            })),
        };

        let arg = match args {
            Ok(v) => v.trim().to_string(),
            Err(e) => {
              let _rep = msg.reply(ctx, &format!("{:?}", e)).await;
              return Ok(());
            }
        };
        
        let is_url = arg.starts_with("http");

        let resolved_src = match is_url {
            true => Restartable::ytdl(arg, true).await,
            false => Restartable::ytdl_search(arg, true).await,
        };
        
        let input = match resolved_src {
            Ok(inp) => Input::from(inp),
            Err(why) => {
              log::error!("Err starting source: {:?}", why);
              let _ = msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await;
              return Ok(());
            }
        };

        let (mut track, _) = create_player(input);
        track.set_volume(0.5);
        handler.set_bitrate(Bitrate::Max);
        handler.enqueue(track);
        let track_queue = handler.queue().clone();

        log::info!("Started playing song");

        handler.add_global_event(
            songbird::Event::Periodic(Duration::from_millis(3000), None),
            DisconnectIfNoUsers::new(manager.clone(), guid, ctx.clone(), connect_to),
        );

        handler.add_global_event(
            songbird::Event::Track(TrackEvent::End), 
            DisconnectIfPlayerQueueEmpty::new(manager.clone(), track_queue.clone(), guild_id),
        );
    }

    log::info!("Joined channel: {}", connect_to);
    Ok(())
}

pub async fn on_skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();

        log::info!("Song skipped to: {} in queue.", queue.len());
    } else {
        log::warn!("Bot not in a voice channel");
    }

    Ok(())
}

pub async fn on_pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialization!!!").clone();

    let handler_lock = manager.get(guild_id).unwrap();

    let handler = handler_lock.lock().await;
    handler.queue().pause().unwrap();

    log::info!("Stopping audio playback");
    Ok(())
}

pub async fn on_leave(ctx: &Context, msg: &Message) -> CommandResult {
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
        log::warn!("Bot not in the voice channel");
    }

    Ok(())
}