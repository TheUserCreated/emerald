use crate::helpers::db::log_set;
use crate::structures::data::{ConnectionPool, LogMap};
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
}

#[command]
#[sub_commands(set)]
pub async fn log(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "You need more arguments.").await?;

    Ok(())
}

#[command]
#[owners_only]
async fn debug(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[required_permissions("MANAGE_GUILD")]
pub async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    log_set(&pool, guild_id, channel_id)
        .await
        .expect("couldn't set log channel");
    let response = format!("Log channel set to {}", channel_id.mention());
    msg.reply(ctx, response).await?;

    Ok(())
}
