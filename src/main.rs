use std::{collections::HashSet, env, sync::Arc};

use serenity::{
    async_trait,
    framework::{
        standard::macros::group,
        standard::macros::hook,
        StandardFramework,
    },
    http::Http,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use serenity::model::prelude::Message;
use tracing::{error, info};
use tracing_subscriber::{
    EnvFilter,
    FmtSubscriber,
};

use commands::config::*;
use commands::meta::*;
use structures::data::*;

use crate::helpers::*;

mod structures;
mod commands;
mod helpers;

struct Handler;

//TODO greetings:
// greetings greet a user when they receive a role
// the channel the greeting takes place in is dependent on the role
// if the user receives another role to be greeted in a short time, in the same channel, it must be edited into the previous greeting
// if many users need to receive the same greeting, make one message but ping many (MAX TEN)


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(ping, prefix, die)]
struct General;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load env file");
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let http = Http::new_with_token(&token);
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };


    #[hook]
    async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
        let data = ctx.data.read().await;

        let prefixes = data.get::<PrefixMap>().cloned();
        let default_prefix =
            env::var("DEFAULT_PREFIX").expect("fuck");


        let guild_id = msg.guild_id;

        match prefixes.unwrap().get(&guild_id?) {
            Some(prefix_guard) => Some(prefix_guard.value().to_owned()),
            None => Some(default_prefix),
        }
    }
    #[hook]
    #[instrument]
    async fn before(_: &Context, msg: &Message, cmd_name: &str) -> bool {
        info!("Got command '{}' by user '{}' with ID {} in guildid {:?}, in channelid {:?}", cmd_name, msg.author.name, msg.author.id, msg.guild_id.unwrap().as_u64(), msg.channel_id.as_u64());

        true
    }
    let framework = StandardFramework::new()
        .configure(|c| {
            c.owners(owners)
                .dynamic_prefix(dynamic_prefix)
                .on_mention(Some(_bot_id))
        })
        .before(before)
        .group(&GENERAL_GROUP);
    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    let pool = db::get_db_pool(env::var("DATABASE_URL").expect("define a database url in env"))
        .await
        .unwrap();
    let prefixes = db::fetch_prefixes(&pool).await.unwrap();
    println!("{:?}", prefixes);

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<ConnectionPool>(pool);
        data.insert::<PrefixMap>(Arc::new(prefixes));
    }
    if let Err(why) = client.start_shards(2).await {
        error!("Client error: {:?}", why);
    }
}