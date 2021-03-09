use dashmap::DashMap;
use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::prelude::*,
    prelude::*,
};
use serenity::static_assertions::_core::str::FromStr;
use tokio::time::{Duration, sleep};
use tracing::{info};

use crate::structures::data::{ChannelMap, ConnectionPool};

#[command]
#[required_permissions("MANAGE_GUILD")]
#[only_in(guilds)]
#[sub_commands(set, check, remove)]
async fn autodelete(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.reply(ctx,"Not enough arguments! Run `help autodelete` to see how to use this command.").await?;
    //let channel_id = args.current().expect("no role found");
    //let channel_id = ChannelId::from_str(channel_id).expect("couldn't unpack roleid from argument");
    //args.advance();
    //let time = args.current().expect("time not found");
    //let time = i64::from_str(time).expect("couldnt get time");

    Ok(())
}
#[command]
#[only_in(guilds)]
#[aliases("delete", "unset")]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult{
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn check(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channelmap = {
        let data = ctx.data.read().await;
        let amnesiacs = data.get::<ChannelMap>().cloned().unwrap();
        amnesiacs

    };

    let channel_id = args.current().expect("no channel found");
    let channel_id = ChannelId::from_str(channel_id).expect("couldn't unpack channelid from argument");
    let result = channelmap.get(&channel_id);
    if let Some(result) = result {
        let minutes = result.value();
        let response = format!("The channel {} has auto-delete enabled, with a timer of {} minute(s).", channel_id.name(ctx).await.expect("channel doesnt have a name?"), minutes);
        msg.reply(ctx,response).await?;

    } else {
        msg.reply(ctx, "That channel doesn't seem to have auto-delete enabled").await?;
    }
    Ok(())
}


#[command]
#[required_permissions("MANAGE_GUILD")]
#[only_in(guilds)]
#[aliases("add")]
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
        let amnesiacs = data.get::<ChannelMap>().cloned().unwrap();

        (pool, amnesiacs)
    };
    amnesiacs.insert(channel_id, time);
    sqlx::query!(
            "INSERT INTO amnesiac_messages (channel_id,duration)\
            VALUES ($2,$1)\
            ON CONFLICT (channel_id) DO UPDATE \
            SET duration = $1;"   ,
            time,
            channel_id.0 as i64
        )
        .execute(&pool)
        .await?;
    let response = format!("Set the message auto-delete timer for {} to {} minutes.", channel_id.name(ctx).await.expect("oops"), time);
    msg.reply(ctx, response).await?;


    Ok(())
}

pub async fn message_handler(ctx: Context, message: Message) {
    let channelmap = {
        let data = ctx.data.read().await;
        let channelmap = data.get::<ChannelMap>().cloned().unwrap();

        channelmap
    };
    let result = channelmap.get(&message.channel_id);
    if let Some(result) = result {
        let minutes = result.value();
        let seconds = minutes*60;
        tokio::spawn(async move {
            sleep(Duration::from_secs(seconds as u64)).await;
            message.delete(ctx).await.expect("couldn't delete message auto-delete area");
        });
    } else {
        return;
    }
}

