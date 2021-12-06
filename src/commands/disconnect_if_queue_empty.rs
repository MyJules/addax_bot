use std::sync::Arc;

use serenity::{model::prelude::*};
use serenity::async_trait;

use songbird::tracks::TrackQueue;
use songbird::{EventContext, Songbird};

pub struct DisconnectIfPlayerQueueEmpty{
    manager: Arc<Songbird>,
    queue: TrackQueue,
    guild_id: GuildId,
}

impl DisconnectIfPlayerQueueEmpty{
    pub fn new(manager: Arc<Songbird>, queue: TrackQueue, guild_id: GuildId) -> Self { Self { manager, queue, guild_id } }

    async fn disconnect_if_queue_empty(&self, ctx: &EventContext<'_>){
        if let EventContext::Track(&[(_, _)]) = ctx{
            if self.queue.is_empty() {
                let _dc = self.manager.leave(self.guild_id).await;
                log::info!("Discord bot disconnected");
            }
        }        
    }
}

#[async_trait]
impl songbird::EventHandler for DisconnectIfPlayerQueueEmpty{
    async fn act(&self, ctx: &EventContext<'_>) -> Option<songbird::Event> {
        self.disconnect_if_queue_empty(ctx).await;
        None
    }
}