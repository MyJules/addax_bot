use std::sync::Arc;

use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::async_trait;

use songbird::tracks::TrackQueue;
use songbird::{EventContext, Songbird};

pub struct DisconnectIfNoUsers {
    manager: Arc<Songbird>,
    guild: Guild,
    ctx: Context,
    queue: TrackQueue,
    connected_to: ChannelId,
}

impl DisconnectIfNoUsers {
    pub fn new(manager: Arc<Songbird>, guild: Guild, ctx: Context, queue: TrackQueue, connected_to: ChannelId) -> Self { Self { manager, guild, ctx, queue, connected_to } }

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
            self.queue.stop();
            let _dc = self.manager.leave(self.guild.id).await;
            log::info!("Discord bot disconnected");
        }
    }
}

#[async_trait]
impl songbird::EventHandler for DisconnectIfNoUsers{
    async fn act(&self, _: &EventContext<'_>) -> Option<songbird::Event> {
        log::info!("Checking if no users left");
        self.disconnect_if_no_users().await;
        None
    }
}