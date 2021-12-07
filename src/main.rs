mod commands;

use std::env;

use simplelog::*;
use log::LevelFilter;

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    StandardFramework,
    macros::group
};

use commands::player::*;
use songbird::{SerenityInit};

#[group]
#[commands(play, skip, pause, leave)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("Connected as {}", ready.user.name)
    }
}

#[tokio::main]
async fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("'"))
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    // Login with a bot token from the environment
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        log::error!("Client runtime error: {:?}", why);
    }
}