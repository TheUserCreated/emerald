use crate::helpers::db::{enable_log_event, log_set, log_update_id};
use crate::structures::data::{ConnectionPool, LogConf, LogMap};
use serenity::static_assertions::_core::str::FromStr;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

pub async fn channel_delete_log(ctx: Context, channel: &GuildChannel) {
    let (pool, logmap) = {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().cloned().unwrap();
        let logmap = data.get::<LogMap>().cloned().unwrap();
        (pool, logmap)
    };
    let guild_id = channel.guild_id;
    let guild_log_config = logmap.get(&guild_id).expect("");
    if !guild_log_config.channel_delete {}
}

pub async fn message_delete_log(
    ctx: Context,
    channel_id: ChannelId,
    message_id: MessageId,
    guild_id: Option<GuildId>,
) {
    let guild_id =
        guild_id.expect("Got message delete event from a guild that no longer exists. Oops.");
    let logmap = {
        let data = ctx.data.read().await;
        let logmap = data.get::<LogMap>().cloned().unwrap();
        logmap
    };
    let guild_log_config = logmap.get(&guild_id).expect(""); //nothing as logged as this isnt an error. guild has no logs configured and thats fine

    if !guild_log_config.message_delete {
        //if message delete logging is off, exit.
        return;
    }
    let log_channel = guild_log_config.log_channel;
    let message = ctx
        .cache
        .message(&channel_id, &message_id)
        .await
        .expect("got a deleted message we couldn't log.");
    if message.author.bot || message.author.id == ctx.cache.current_user().await.id {
        //we don't log messages from us or other bots
        return;
    }
    let log_channel = ChannelId::from(log_channel);
    let author_id = message.author.id;
    log_channel
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Deleted Message:");
                e.author(|a| {
                    a.icon_url(message.author.avatar_url().unwrap_or_else(|| {
                        "https://cdn.discordapp.com/embed/avatars/0.png"
                            .parse()
                            .unwrap()
                    }));
                    a.name(&message.author.name);
                    a
                });
                e.timestamp(message.timestamp.naive_utc().to_string());
                e.description(&message.content);
                e.footer(|f| {
                    f.text(author_id);
                    f
                });
                e
            });
            m
        })
        .await
        .expect("couldnt send log message. do i lack perms?");
}

#[command]
#[sub_commands(set, add)]
pub async fn log(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "You need more arguments.").await?;

    Ok(())
}

#[command]
#[required_permissions("MANAGE_GUILD")]
pub async fn set(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let channel_id = args.current().expect("no channel found");
    let channel_id =
        ChannelId::from_str(channel_id).expect("couldn't unpack channelid from argument");
    let (pool, logmap) = {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().cloned().unwrap();
        let logmap = data.get::<LogMap>().cloned().unwrap();

        (pool, logmap)
    };
    if logmap.get(&guild_id).is_some() {
        log_update_id(&pool, guild_id, channel_id)
            .await
            .expect("couldn't set log channel");
        logmap.alter(&guild_id, |_, mut v| {
            v.log_channel = channel_id.0;
            v
        })
    } else {
        log_set(&pool, guild_id, channel_id)
            .await
            .expect("couldn't set log channel");
        let conf = LogConf {
            log_channel: channel_id.0,
            channel_create: false,
            channel_update: false,
            channel_delete: false,
            ban_add: false,
            ban_remove: false,
            member_join: false,
            member_remove: false,
            role_create: false,
            role_update: false,
            role_delete: false,
            invite_create: false,
            invite_delete: false,
            message_edit: false,
            message_delete: false,
            message_delete_bulk: false,
            webhook_update: false,
        };
        logmap.insert(guild_id, conf);
    }

    let response = format!("Log channel set to {}", channel_id.mention());
    msg.reply(ctx, response).await?;

    Ok(())
}
#[command]
#[required_permissions("MANAGE_GUILD")]
pub async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let log_id: u64 = args.parse().expect("bad arguments");
    let (pool, logmap) = {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().cloned().unwrap();
        let logmap = data.get::<LogMap>().cloned().unwrap();

        (pool, logmap)
    };
    if log_id == 13 {
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.message_delete = true;
                v
            },
        );
        enable_log_event(
            &pool,
            msg.guild_id.expect("message from deleted guild"),
            13,
            true,
        )
        .await?;
    }

    Ok(())
}
