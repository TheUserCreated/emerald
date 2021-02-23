use dashmap::DashMap;
use serenity::{framework::standard::CommandResult, model::id::GuildId};
use sqlx::postgres::{PgPool, PgPoolOptions};
use serenity::model::id::{ChannelId, RoleId};
use tracing::{error, info};

pub async fn get_db_pool(db_connection: String) -> CommandResult<PgPool> {
    let connection_string = &db_connection;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

    Ok(pool)
}

pub async fn fetch_prefixes(pool: &PgPool) -> CommandResult<DashMap<GuildId, String>> {
    let prefixes: DashMap<GuildId, String> = DashMap::new();

    let cursor = sqlx::query!("SELECT guild_id, prefix FROM guild_info")
        .fetch_all(pool)
        .await?;

    for i in cursor {
        if let Some(prefix) = i.prefix {
            prefixes.insert(GuildId::from(i.guild_id as u64), prefix);
        }
    }
    Ok(prefixes)
}

//pub async fn fetch_greeting(pool: &PgPool, guild_id: &GuildId, channel_id: ChannelId,role_id: RoleId) -> CommandResult<String> {
//    let greeting = sqlx::query!("SELECT greeting WHERE guild_id = $1",
//    guild_id)
//    Ok(("fuckoff".parse().unwrap()))
//}

pub async fn set_greeting_internal(pool: &PgPool, guild_id: &GuildId, channel_id: ChannelId,role_id: RoleId, greeting_text: String) -> CommandResult{

    sqlx::query!(
            "INSERT INTO greeting_info (guild_id,channel_id,role_id,greeting)\
            VALUES ($1,$2,$3,$4)\
            ON CONFLICT (guild_id) DO UPDATE \
            SET greeting = $4;"   ,
            guild_id.0 as i64,
            channel_id.0 as i64,
            role_id.0 as i64,
            greeting_text

        )
            .execute(pool)
            .await?;
    Ok(())
}