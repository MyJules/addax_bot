use std::time::Duration;
use songbird::driver::Bitrate;
use serenity::framework::standard::{Args};
use songbird::tracks::{PlayMode};
use songbird::{create_player, TrackEvent};
use songbird::input::{Input, Restartable};

use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::CommandResult;

use crate::commands::disconnect_if_no_user_left::DisconnectIfNoUsers;
use crate::commands::disconnect_if_queue_empty::DisconnectIfPlayerQueueEmpty;

pub async fn bot_play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    //parse args
    let args = match args.len() {
        0 => {
            resume_play(ctx, msg).await.unwrap();
            Err("Zero arguments")
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
        Err(_) => {
        return Ok(());
        }
    };

    //join
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialization!!!").clone();
    let (handler_lock, is_connected) = manager.join(guild_id, connect_to).await;
    
    if let Ok(_) = is_connected {
        
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
        
        let mut handler = handler_lock.lock().await;
        let (mut track, _) = create_player(input);
        track.set_volume(0.5);
        handler.set_bitrate(Bitrate::Max);
        handler.enqueue(track);
        log::info!("Started playing song");
        
        let track_queue = handler.queue().clone();

        handler.add_global_event(
            songbird::Event::Periodic(Duration::from_secs(3), None),
            DisconnectIfNoUsers::new(manager.clone(), guid, ctx.clone(), track_queue.clone(), connect_to),
        );
        
        handler.add_global_event(
            songbird::Event::Track(TrackEvent::End), 
            DisconnectIfPlayerQueueEmpty::new(manager.clone(), track_queue.clone(), guild_id),
        );
    }

    log::info!("Joined channel: {}", connect_to);
    Ok(())
}

pub async fn resume_play(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialization!!!").clone();
    
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
    
        if handler.queue().is_empty() {
            log::warn!("No tracks in playlist");
            let _ = msg.channel_id.say(&ctx.http, "Sonya have nothing to sing.").await;
            return Ok(());
        }

        log::info!("Resuming track");
        let _ = match handler.queue().resume() {
            Ok(handler) => handler,
            Err(error) => log::error!("Resume error: {}", error),
        };
        
        log::info!("Bot resumed playing song");
    } else {
        msg.channel_id.say(&ctx.http, "Sonya have nothing to sing.").await.unwrap();
        log::warn!("Bot not in the voice channel");
    }

    Ok(())
}

pub async fn bot_skip(ctx: &Context, msg: &Message) -> CommandResult {
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
        msg.channel_id.say(&ctx.http, "Sonya not in the voice channel.").await.unwrap();
        log::warn!("Bot not in the voice channel");
    }

    Ok(())
}

pub async fn bot_pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialization!!!").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        
        if let Some(track) = handler.queue().current(){
            let track_state = track.get_info().await.unwrap();
        
            match track_state.playing {
                PlayMode::Play => handler.queue().pause().unwrap(),
                PlayMode::Pause => {
                    msg.channel_id.say(&ctx.http, "Sonya is already waiting for you to continue listening.").await.unwrap();
                    ()
                },
                _ => (),
            }
        }else{
            msg.channel_id.say(&ctx.http, "Sonya not in the voice channel.").await.unwrap();
        }

        log::info!("Stopping audio playback");
    }else{
        msg.channel_id.say(&ctx.http, "Sonya not in the voice channel.").await.unwrap();
        log::warn!("Bot not in the voice channel");
    }
    Ok(())
}

pub async fn bot_leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialization.").clone();
    
    if let Some(handler_lock) = manager.get(guild_id) {
        if let Err(e) = manager.remove(guild_id).await {
            log::error!("Error disconnecting from voice channel: {:?}", e);
        }
        let handler = handler_lock.lock().await;
        handler.queue().stop();
        log::info!("Bot left channel");
    }else{
        msg.channel_id.say(&ctx.http, "Sonya not in the voice channel.").await.unwrap();
        log::warn!("Bot not in the voice channel");
    }

    Ok(())
}

pub async fn bot_print_help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http,
        "Print help!!!"
    ).await.unwrap();

    Ok(())
}