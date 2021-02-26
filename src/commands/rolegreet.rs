use std::collections::HashSet;

use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::prelude::*,
    prelude::*,
};
use serenity::static_assertions::_core::str::FromStr;
use tracing::{debug};

use crate::db::remove_greeting_internal;
use crate::db::set_greeting_internal;
use crate::helpers::db::get_greeting;
use crate::structures::data::ConnectionPool;

#[command]
#[required_permissions("MANAGE_GUILD")]
#[sub_commands(remove, add)]
async fn greeting(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    msg.reply(ctx, "Use add, remove, or list.").await?;

    Ok(())
}
#[command]
#[required_permissions("MANAGE_GUILD")]
async fn add(ctx: &Context, msg:&Message, mut args: Args) -> CommandResult{
    let role = args.current().expect("no role found");
    let role = RoleId::from_str(role).expect("couldn't unpack roleid from argument");
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().cloned().unwrap();
    let guild_id = msg.guild_id.expect("got a message from a guild that doesnt exist anymore");
    let channel_id = msg.channel_id;
    args.advance();
    let greeting = args.rest();
    set_greeting_internal(&pool, &guild_id, channel_id, role, greeting.parse().unwrap()).await?;
    Ok(())
}

#[command]
#[required_permissions("MANAGE_GUILD")]
async fn remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let role = args.current().expect("no role found");
    let role = RoleId::from_str(role).expect("couldn't unpack roleid from argument");
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().cloned().unwrap();
    let guild_id = msg.guild_id.expect("got a message from a guild that doesnt exist anymore");
    let channel_id = msg.channel_id;
    remove_greeting_internal(&pool, &guild_id, &channel_id, &role).await?;
    Ok(())
}


pub async fn greeting_handler(ctx: Context, old_if_available: Option<Member>, new: Member) -> CommandResult {
    let old = match old_if_available {
        Some(m) => m,
        None => return Ok(()),
    };

    let oldroles = &old.roles;
    let newroles = &new.roles;

    let role_set: HashSet<_> = oldroles.iter().collect();
    let difference: Vec<_> = newroles.iter().filter(|item| !role_set.contains(item)).collect();
    if difference.is_empty() {
        return Ok(());
    }

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().cloned().unwrap();
    let greet_result = get_greeting(&pool, &new.guild_id, **difference.first().unwrap()).await;
    let (channel_id, greeting) = greet_result.expect("couldn't get greeting data from database");
    let greeting = greeting_replacements(&ctx, &new, greeting).await.expect("failure at greeting text replacement!");
    if channel_id.0 == 0 {
        return Ok(())
    }
    debug!("greeting attempting to be sent in channel {:?}",channel_id);
    channel_id.say(ctx.http, greeting).await.expect("couldn't send the greeting. do i have perms?");
    Ok(())
}

pub async fn greeting_replacements(ctx: &Context, new: &Member, text: String) -> CommandResult<String> {
    let guild_name = new.guild_id.name(&ctx).await.expect("couldn't get guild name to replace");
    let finished_text = text.replace("%servername%", &*guild_name);
    let finished_text = finished_text.replace("%usermention%", &*new.mention().to_string());
    Ok(finished_text)
}