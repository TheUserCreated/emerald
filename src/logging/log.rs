use crate::helpers::db::{enable_log_event, log_set, log_update_id};
use crate::structures::data::{ConnectionPool, LogConf, LogMap};
use chrono::Utc;
use serenity::static_assertions::_core::str::FromStr;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
pub async fn channel_create_log(ctx: Context, channel: &GuildChannel) {
    let logmap = {
        let data = ctx.data.read().await;
        let logmap = data.get::<LogMap>().cloned().unwrap();
        logmap
    };
    let guild_id = channel.guild_id;
    let guild_log_config = logmap.get(&guild_id).expect("");
    if !guild_log_config.channel_create {
        return;
    }
    let log_channel = ChannelId::from(guild_log_config.log_channel);
    let guild = Guild::get(&ctx.http, channel.guild_id)
        .await
        .expect("log from a guild im not in?");
    let response = format!(
        "Channel {} with ID {} was created.",
        channel.name, channel.id
    );
    log_channel
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Channel Created:");
                e.colour(Colour::BLUE);
                e.author(|a| {
                    let url = guild.icon_url();
                    if url.is_none() {
                        return a;
                    };
                    a.icon_url(url.unwrap());
                    a
                });
                e.description(response);
                e.timestamp(&Utc::now());
                e
            });

            m
        })
        .await
        .expect("Couldn't send log message. Do I lack perms?");
}

pub async fn channel_delete_log(ctx: Context, channel: &GuildChannel) {
    let logmap = {
        let data = ctx.data.read().await;
        let logmap = data.get::<LogMap>().cloned().unwrap();
        logmap
    };
    let guild_id = channel.guild_id;
    let guild_log_config = logmap.get(&guild_id).expect("");
    if !guild_log_config.channel_delete {
        return;
    }
    let log_channel = ChannelId::from(guild_log_config.log_channel);
    let guild = Guild::get(&ctx.http, channel.guild_id)
        .await
        .expect("log from a guild im not in?");
    let response = format!(
        "Channel {} with ID {} was deleted.",
        channel.name, channel.id
    );
    log_channel
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Channel Deleted:");
                e.colour(Colour::RED);
                e.author(|a| {
                    let url = guild.icon_url();
                    if url.is_none() {
                        return a;
                    };
                    a.icon_url(url.unwrap());
                    a.name(guild.name);
                    a
                });
                e.description(response);
                e.timestamp(&Utc::now());
                e
            });

            m
        })
        .await
        .expect("Couldn't send log message. Do I lack perms?");
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
    let audit_log = ctx
        .http
        .get_audit_logs(guild_id.0, Some(72), None, None, Some(1))
        .await
        .unwrap();
    for (_entry_id, entry) in audit_log.entries {
        if entry.target_id.is_some() && entry.target_id.unwrap() == message.author.id.0 {
            let executor = entry.user_id;
            let executor = ctx
                .http
                .get_user(executor.0)
                .await
                .expect("couldn't get executor in logging");
            let log_channel = ChannelId::from(log_channel);
            let author_id = message.author.id;
            let executor_name = executor.name;
            let channel = ctx
                .http
                .get_channel(message.channel_id.0)
                .await
                .expect("couldn't get channel in logging");
            let channel_name = channel.mention().to_string();
            let title = format!("Message deleted by {}:", executor_name);
            log_channel
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(title);
                        e.colour(Colour::RED);
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
                .expect("couldn't send log message. do i lack perms?");
            return;
        } else {
            let log_channel = ChannelId::from(log_channel);
            let author_id = message.author.id;
            log_channel
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Message Deleted:");
                        e.colour(Colour::RED);
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
                .expect("couldn't send log message. do i lack perms?");
        }
    }
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
    let mut response = "";
    let mut event = 0;
    if log_id == 1 {
        response = "channel create";
        event = 1;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.channel_create = true;
                v
            },
        );
    } else if log_id == 2 {
        response = "channel update";
        event = 2;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.channel_update = true;
                v
            },
        );
    } else if log_id == 3 {
        response = "channel delete";
        event = 3;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.channel_delete = true;
                v
            },
        );
    } else if log_id == 4 {
        response = "ban add";
        event = 4;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.ban_add = true;
                v
            },
        );
    } else if log_id == 5 {
        response = "ban remove";
        event = 1;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.ban_remove = true;
                v
            },
        );
    } else if log_id == 6 {
        response = "member_join";
        event = 6;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.member_join = true;
                v
            },
        );
    } else if log_id == 7 {
        response = "member remove";
        event = 7;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.member_remove = true;
                v
            },
        );
    } else if log_id == 8 {
        response = "role create";
        event = 8;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.role_create = true;
                v
            },
        );
    } else if log_id == 9 {
        response = "role update";
        event = 9;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.role_update = true;
                v
            },
        );
    } else if log_id == 10 {
        response = "role delete";
        event = 10;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.role_delete = true;
                v
            },
        );
    } else if log_id == 11 {
        response = "invite create";
        event = 11;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.invite_create = true;
                v
            },
        );
    } else if log_id == 12 {
        response = "invite delete";
        event = 12;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.invite_delete = true;
                v
            },
        );
    } else if log_id == 13 {
        response = "message edit";
        event = 13;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.message_edit = true;
                v
            },
        );
    } else if log_id == 14 {
        response = "message delete";
        event = 14;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.message_delete = true;
                v
            },
        );
    } else if log_id == 15 {
        response = "message bulk delete";
        event = 15;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.message_delete_bulk = true;
                v
            },
        );
    } else if log_id == 16 {
        response = "webhook update";
        event = 16;
        logmap.alter(
            &msg.guild_id.expect("command from guild that doesnt exist"),
            |_, mut v| {
                v.webhook_update = true;
                v
            },
        );
    }
    enable_log_event(
        &pool,
        msg.guild_id.expect("message from deleted guild"),
        event,
        true,
    )
    .await
    .expect("couldnt add log event.");
    let response = format!("The event `{}` will now be logged.", response);
    msg.reply(ctx, response).await?;
    Ok(())
}
