use crate::structures::data::LogConf;
use dashmap::DashMap;
use serenity::model::id::{ChannelId, RoleId};
use serenity::{framework::standard::CommandResult, model::id::GuildId};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

pub async fn get_db_pool(db_connection: String) -> CommandResult<PgPool> {
    let connection_string = &db_connection;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

    Ok(pool)
}
pub async fn fetch_logdata(pool: &PgPool) -> CommandResult<DashMap<GuildId, LogConf>> {
    let logmap: DashMap<GuildId, LogConf> = DashMap::new();
    let cursor = sqlx::query!("SELECT * FROM logging;")
        .fetch_all(pool)
        .await?;
    for element in cursor {
        let guild_id = GuildId::from(element.guild_id as u64);
        let conf = LogConf {
            log_channel: element.channel_id as u64,
            channel_create: element.channel_create as bool,
            channel_update: element.channel_update as bool,
            channel_delete: element.channel_delete as bool,
            ban_add: element.ban_add as bool,
            ban_remove: element.ban_remove as bool,
            member_join: element.member_join as bool,
            member_remove: element.member_remove as bool,
            role_create: element.role_create as bool,
            role_update: element.role_update as bool,
            role_delete: element.role_delete as bool,
            invite_create: element.invite_create as bool,
            invite_delete: element.invite_delete as bool,
            message_edit: element.message_edit as bool,
            message_delete: element.message_delete as bool,
            message_delete_bulk: element.message_delete_bulk as bool,
            webhook_update: element.webhook_update as bool,
        };
        logmap.insert(guild_id, conf);
    }
    Ok(logmap)
}
pub async fn log_update_id(
    pool: &PgPool,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> CommandResult {
    sqlx::query!(
        "UPDATE logging \
        SET channel_id = $1\
        WHERE guild_id = $2",
        channel_id.0 as i64,
        guild_id.0 as i64
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn log_set(pool: &PgPool, guild_id: GuildId, channel_id: ChannelId) -> CommandResult {
    sqlx::query!("INSERT INTO logging (guild_id,channel_id,\
    channel_create,channel_update,channel_delete,ban_add,ban_remove,member_join,member_remove,role_create,\
    role_update,role_delete,invite_create,invite_delete,message_edit,message_delete,message_delete_bulk,webhook_update)\
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)",
            guild_id.0 as i64,
            channel_id.0 as i64,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            ).execute(pool).await?;

    Ok(())
}

pub async fn enable_log_event(
    pool: &PgPool,
    guild_id: GuildId,
    event: u8,
    value: bool,
) -> CommandResult {
    match event {
        1 => {}
        2 => {}
        3 => {
            sqlx::query!(
                "UPDATE logging \
        SET channel_delete = $1 \
        WHERE guild_id = $2",
                value,
                guild_id.0 as i64
            )
            .execute(pool)
            .await?;
        }
        4 => {}
        5 => {}
        6 => {}
        7 => {}
        8 => {}
        9 => {}
        10 => {}
        11 => {}
        12 => {}
        13 => {}
        14 => {
            sqlx::query!(
                "UPDATE logging \
        SET message_delete = $1 \
        WHERE guild_id = $2",
                value,
                guild_id.0 as i64
            )
            .execute(pool)
            .await?;
        }
        15 => {}
        16 => {}
        _ => {
            info!("tried to enable a log event that doesnt exist?")
        }
    }

    Ok(())
}
//internal representation of log IDs
/*
   channel_create 1
   channel_update 2
   channel_delete 3
   ban_add 4
   ban_remove 5
   member_join 6
   member_remove 7
   role_create 8
   role_update 9
   role_delete 10
   invite_create 11
   invite_delete 12
   message_edit 13
   message_delete 14
   message_delete_bulk 15
   webhook_update 16
*/

pub async fn fetch_amnesiacs(pool: &PgPool) -> CommandResult<DashMap<ChannelId, i64>> {
    let channelmap: DashMap<ChannelId, i64> = DashMap::new();
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

pub async fn delete_amnesiac(pool: &PgPool, channel_id: ChannelId) -> CommandResult {
    sqlx::query!(
        "DELETE FROM amnesiac_messages WHERE channel_id = $1",
        channel_id.0 as i64
    )
    .execute(pool)
    .await?;

    Ok(())
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

pub async fn set_greeting_internal(
    pool: &PgPool,
    guild_id: &GuildId,
    channel_id: ChannelId,
    role_id: RoleId,
    greeting_text: String,
) -> CommandResult {
    sqlx::query!(
        "INSERT INTO greeting_info (guild_id,channel_id,role_id,greeting)\
            VALUES ($1,$2,$3,$4)\
            ON CONFLICT (guild_id) DO UPDATE \
            SET greeting = $4;",
        guild_id.0 as i64,
        channel_id.0 as i64,
        role_id.0 as i64,
        greeting_text
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_greeting(
    pool: &PgPool,
    guild_id: &GuildId,
    role_id: &RoleId,
) -> CommandResult<(ChannelId, String)> {
    let cursor = sqlx::query!(
        "SELECT channel_id, greeting FROM greeting_info WHERE guild_id = $1 AND role_id = $2",
        guild_id.0 as i64,
        role_id.0 as i64
    )
    .fetch_all(pool)
    .await?;

    let mut channel_id: i64 = 0;
    let mut greeting: String = "".to_string();
    for items in cursor {
        channel_id = items.channel_id;
        greeting = items.greeting;
    }
    let channel_id = channel_id as u64;

    return Ok((ChannelId::from(channel_id), greeting));
}

pub async fn remove_greeting_internal(
    pool: &PgPool,
    guild_id: &GuildId,
    channel_id: &ChannelId,
    role_id: &RoleId,
) -> CommandResult {
    sqlx::query!(
        "DELETE FROM greeting_info WHERE guild_id = $1 AND channel_id = $2 AND role_id = $3",
        guild_id.0 as i64,
        channel_id.0 as i64,
        role_id.0 as i64,
    )
    .execute(pool)
    .await?;
    Ok(())
}

//NOTE: it seems ill have to re-implement removal functions for all situations that need to trigger a deletion
//      there might be a cleaner way to do this using generics, if i figure that out expect these functions to go bye-bye
pub async fn remove_greeting_by_channel(pool: &PgPool, channel_id: &ChannelId) -> CommandResult {
    sqlx::query!(
        "DELETE FROM greeting_info WHERE channel_id = $1 ",
        channel_id.0 as i64,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_greeting_by_role(pool: &PgPool, role_id: &RoleId) -> CommandResult {
    sqlx::query!(
        "DELETE FROM greeting_info WHERE role_id = $1",
        role_id.0 as i64,
    )
    .execute(pool)
    .await?;
    Ok(())
}
