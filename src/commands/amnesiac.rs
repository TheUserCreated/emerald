use serenity::static_assertions::_core::str::FromStr;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use tokio::time::{sleep, Duration};
use tracing::info;

use crate::helpers::db::delete_amnesiac;
use crate::structures::data::{AmnesiaMap, ConnectionPool};
use dashmap::DashMap;
use serenity::builder::CreateMessage;
use serenity_utils::menu::*;
use serenity_utils::menu::{Menu, MenuOptions};
use std::sync::Arc;

#[command]
#[required_permissions("MANAGE_GUILD")]
#[only_in(guilds)]
#[sub_commands(set, list, remove)]
#[description = "Set of commands related to automatically deleting messages sent in channels. Cannot be used without a sub-command"]
async fn autodelete(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    msg.reply(
        ctx,
        "Invalid arguments! Run `help autodelete` to see how to use this command.",
    )
    .await?;
    //let channel_id = args.current().expect("no role found");
    //let channel_id = ChannelId::from_str(channel_id).expect("couldn't unpack roleid from argument");
    //args.advance();
    //let time = args.current().expect("time not found");
    //let time = i64::from_str(time).expect("couldnt get time");

    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_CHANNELS")]
#[description = "Lists all channels in this server that have auto-delete enabled."]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let channel_list = msg
        .guild(ctx)
        .await
        .expect("got a message from a guild that doesn't exist")
        .channels;
    let mut channels: Vec<ChannelId> = Vec::with_capacity(500);

    for i in channel_list.into_keys() {
        channels.push(i);
    }
    let amnesiacs = {
        let data = ctx.data.read().await;

        let amnesiacs = data.get::<AmnesiaMap>().cloned().unwrap();

        amnesiacs
    };
    let relevant_channels: DashMap<ChannelId, u64> = DashMap::with_capacity(500);
    for i in channels {
        let result = amnesiacs.get(&i);
        if let Some(result) = result {
            relevant_channels.insert(i, *result.value() as u64); // this requires a copy of `value`, sad.
        }
    }

    let controls = vec![
        Control::new(
            ReactionType::from('◀'),
            Arc::new(|m, r| Box::pin(prev_page(m, r))),
        ),
        Control::new(
            ReactionType::from('❌'),
            Arc::new(|m, r| Box::pin(close_menu(m, r))),
        ),
        Control::new(
            ReactionType::from('▶'),
            Arc::new(|m, r| Box::pin(next_page(m, r))),
        ),
    ];
    let options = MenuOptions {
        controls,
        ..Default::default()
    };
    let mut pages: Vec<CreateMessage> = Vec::new();
    for (key, value) in relevant_channels.into_iter() {
        let channel_name = ChannelId(key.0)
            .name(ctx)
            .await
            .expect("channel without a name");
        let mut page = CreateMessage::default();
        let response = format!(
            "Channel {} has auto-delete set for {} minute(s).",
            ChannelId(key.0).mention(),
            value
        );

        page.embed(|e| {
            e.description(response);
            e.title(channel_name);
            e
        });
        pages.push(page);
    }

    let menu = Menu::new(ctx, msg, &pages.as_ref(), options);
    let _ = menu.run().await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases("delete", "unset")]
#[required_permissions("MANAGE_GUILD")]
#[usage = "#channelname"]
#[description = "Removes auto-delete from a channel."]
async fn remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (pool, amnesiacs) = {
        let data = ctx.data.read().await;

        let pool = data.get::<ConnectionPool>().cloned().unwrap();
        let amnesiacs = data.get::<AmnesiaMap>().cloned().unwrap();

        (pool, amnesiacs)
    };
    let channel_id = args.current().expect("no channel found");
    let channel_id =
        ChannelId::from_str(channel_id).expect("couldn't unpack channelid from argument");
    delete_amnesiac(&pool, channel_id).await?;
    amnesiacs.remove(&channel_id);
    msg.reply(ctx, "Auto-delete removed for specified channel.")
        .await?;

    Ok(())
}

#[command]
#[required_permissions("MANAGE_GUILD")]
#[only_in(guilds)]
#[aliases("add")]
#[description = "Enables auto-delete for a channel, with the amount of time messages exist specified in minutes."]
#[usage = "#ChannelName 10 (sets #ChannelName to have messages deleted after 10 minutes"]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel_id = args.current().expect("no role found");
    let channel_id = ChannelId::from_str(channel_id).expect("couldn't unpack roleid from argument");
    args.advance();
    let time = args.current().expect("time not found");
    let time = i64::from_str(time).expect("couldnt get time");
    info!("role is {} and time is {}", channel_id, time);
    let (pool, amnesiacs) = {
        let data = ctx.data.read().await;

        let pool = data.get::<ConnectionPool>().cloned().unwrap();
        let amnesiacs = data.get::<AmnesiaMap>().cloned().unwrap();

        (pool, amnesiacs)
    };
    amnesiacs.insert(channel_id, time);
    sqlx::query!(
        "INSERT INTO amnesiac_messages (channel_id,duration)\
            VALUES ($2,$1)\
            ON CONFLICT (channel_id) DO UPDATE \
            SET duration = $1;",
        time,
        channel_id.0 as i64
    )
    .execute(&pool)
    .await?;
    let response = format!(
        "Set the message auto-delete timer for {} to {} minutes.",
        channel_id.name(ctx).await.expect("oops"),
        time
    );
    msg.reply(ctx, response).await?;

    Ok(())
}

pub async fn message_handler(ctx: Context, message: Message) {
    let channelmap = {
        let data = ctx.data.read().await;
        let channelmap = data.get::<AmnesiaMap>().cloned().unwrap();

        channelmap
    };
    let result = channelmap.get(&message.channel_id);
    if let Some(result) = result {
        let minutes = result.value();
        let seconds = minutes * 60;
        tokio::spawn(async move {
            sleep(Duration::from_secs(seconds as u64)).await;
            message
                .delete(ctx)
                .await
                .expect("couldn't delete message auto-delete area");
        });
    } else {
        return;
    }
}
