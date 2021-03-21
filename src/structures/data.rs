use std::sync::Arc;

use dashmap::DashMap;
use serenity::{
    client::bridge::gateway::ShardManager,
    model::id::{GuildId, MessageId},
    prelude::{Mutex, TypeMapKey},
};
use sqlx::PgPool;

use serenity::model::id::ChannelId;

pub struct PrefixMap;
pub struct GreetMap;
pub struct AmnesiaMap;
impl TypeMapKey for PrefixMap {
    type Value = Arc<DashMap<GuildId, String>>;
}

impl TypeMapKey for GreetMap {
    type Value = Arc<DashMap<ChannelId, MessageId>>;
}
impl TypeMapKey for AmnesiaMap {
    type Value = Arc<DashMap<ChannelId, i64>>;
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}
