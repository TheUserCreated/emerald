use dashmap::DashMap;
use serenity::{framework::standard::CommandResult, model::id::GuildId};
use sqlx::postgres::{PgPool, PgPoolOptions};
use serenity::model::id::{ChannelId, RoleId};


pub async fn get_db_pool(db_connection: String) -> CommandResult<PgPool> {
    let connection_string = &db_connection;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

    Ok(pool)
}
pub async fn fetch_amnesiacs(pool: &PgPool )-> CommandResult<DashMap<ChannelId,i64>>{
    let channelmap:DashMap<ChannelId,i64> = DashMap::new();
    let cursor = sqlx::query!("SELECT channel_id,duration FROM amnesiac_messages")
        .fetch_all(pool)
        .await?;

    for i in cursor {
        if let Some(duration) = i.duration {
            channelmap.insert(ChannelId::from(i.channel_id as u64), duration);
        }
    }
    Ok(channelmap)

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

pub async fn get_greeting(pool: &PgPool, guild_id: &GuildId, role_id: &RoleId) -> CommandResult<(ChannelId, String)>{
     let cursor = sqlx::query!("SELECT channel_id, greeting FROM greeting_info WHERE guild_id = $1 AND role_id = $2",
            guild_id.0 as i64,
            role_id.0 as i64
    ).fetch_all(pool).await?;

    let mut channel_id: i64 = 0;
    let mut greeting: String = "".to_string();
    for items in cursor{
        channel_id = items.channel_id;
        greeting = items.greeting;
    }
    let channel_id= channel_id as u64;

    return Ok((ChannelId::from(channel_id), greeting))


}

pub async fn remove_greeting_internal(pool: &PgPool, guild_id: &GuildId, channel_id: &ChannelId,role_id: &RoleId) -> CommandResult{
    sqlx::query!("DELETE FROM greeting_info WHERE guild_id = $1 AND channel_id = $2 AND role_id = $3",
    guild_id.0 as i64,
    channel_id.0 as i64,
    role_id.0 as i64,
    )
        .execute(pool)
        .await?;
    Ok(())
}