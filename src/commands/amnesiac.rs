use dashmap::DashMap;
use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::prelude::*,
    prelude::*,
};
use serenity::static_assertions::_core::str::FromStr;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

use crate::structures::data::{ChannelMap, ConnectionPool};

#[command]
#[required_permissions("MANAGE_GUILD")]
#[only_in(guilds)]
#[sub_commands(set)]
async fn autodelete(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel_id = args.current().expect("no role found");
    let channel_id = ChannelId::from_str(channel_id).expect("couldn't unpack roleid from argument");
    args.advance();
    let time = args.current().expect("time not found");
    let time = i64::from_str(time).expect("couldnt get time");
    info!("role is {} and time is {}", channel_id, time);


    Ok(())
}

#[command]
#[required_permissions("MANAGE_GUILD")]
#[only_in(guilds)]
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

