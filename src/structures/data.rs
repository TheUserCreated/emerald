use std::{sync::Arc};

use dashmap::DashMap;
use serenity::{
    client::bridge::gateway::ShardManager,
    model::id::{GuildId},
    prelude::{Mutex, TypeMapKey},
};
use sqlx::PgPool;

pub struct PrefixMap;

impl TypeMapKey for PrefixMap {
    type Value = Arc<DashMap<GuildId, String>>;
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}