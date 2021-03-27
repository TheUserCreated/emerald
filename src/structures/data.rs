use std::sync::Arc;

use dashmap::DashMap;
use serenity::{
    client::bridge::gateway::ShardManager,
    model::id::{GuildId, MessageId},
    prelude::{Mutex, TypeMapKey},
};
use sqlx::PgPool;

use serenity::model::id::ChannelId;
pub struct ShardManagerContainer;
pub struct PrefixMap;
pub struct GreetMap;
pub struct AmnesiaMap;
pub struct GuildChannelMap;
pub struct LogMap;
//this could all be done in redis for free improvement

impl TypeMapKey for GuildChannelMap {
    type Value = Arc<DashMap<GuildId, ChannelId>>;
}
impl TypeMapKey for LogMap {
    type Value = Arc<DashMap<GuildId, LogConf>>;
}
impl TypeMapKey for PrefixMap {
    type Value = Arc<DashMap<GuildId, String>>;
}

impl TypeMapKey for GreetMap {
    type Value = Arc<DashMap<ChannelId, MessageId>>;
}
impl TypeMapKey for AmnesiaMap {
    type Value = Arc<DashMap<ChannelId, i64>>;
}

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}
pub struct LogConf {
    pub log_channel: u64,
    pub channel_create: bool,
    pub channel_update: bool,
    pub channel_delete: bool,
    pub ban_add: bool,
    pub ban_remove: bool,
    pub member_join: bool,
    pub member_remove: bool,
    pub role_create: bool,
    pub role_update: bool,
    pub role_delete: bool,
    pub invite_create: bool,
    pub invite_delete: bool,
    pub message_edit: bool,
    pub message_delete: bool,
    pub message_delete_bulk: bool,
    pub webhook_update: bool,
}
