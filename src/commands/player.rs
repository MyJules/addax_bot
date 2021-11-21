use std::sync::Arc;
use std::time::Duration;

use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::async_trait;

use songbird::{EventContext, Songbird};

struct DisconnectHandler {
    manager: Arc<Songbird>,
    guild: Guild,
    ctx: Context,
    connected_to: ChannelId,
}

impl DisconnectHandler {
    fn new(manager: Arc<Songbird>, guild: Guild, ctx: Context, connected_to: ChannelId) -> Self { Self { manager, guild, ctx, connected_to } }

    async fn disconnect_if_no_users(&self){
        let should_disconnect: bool = match self.manager.get(self.guild.id) {
            Some(_) => {        
                let voice_channel = self.guild.channels.get(&self.connected_to).unwrap();
                let members = voice_channel.members(&self.ctx.cache).await.unwrap();
                log::info!("Voice members count: {}", members.len());
                members.len() <= 1
            },
            None => {
                log::warn!("Bot not in the voice channel");
                false
            },
        };

        if should_disconnect {
            let _dc = self.manager.leave(self.guild.id).await;
            log::info!("Discord bot disconnected");
        }
    }
}

#[async_trait]
impl songbird::EventHandler for DisconnectHandler{
    async fn act(&self, ctx: &EventContext<'_>) -> Option<songbird::Event> {
        log::info!("Checking if no users left");
        self.disconnect_if_no_users().await;
        None
    }
}

#[command]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    connect(ctx, msg).await.unwrap();
    log::info!("Start playing music");
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    log::info!("Pause playing music");
    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
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