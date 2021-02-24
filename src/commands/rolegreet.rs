use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::prelude::*,
    prelude::*,
};
use tracing::{info};
use crate::helpers::perms::greetperms;
use crate::structures::data::{ConnectionPool};
use serenity::static_assertions::_core::str::FromStr;
use crate::db::set_greeting_internal;
use std::collections::HashSet;
use crate::helpers::db::get_greeting;


#[command]
#[required_permissions("MANAGE_GUILD")]
async fn set_greeting(ctx: &Context, msg:&Message, mut args: Args) -> CommandResult{
    let role= args.current().expect("no role found");
    let role = RoleId::from_str(role).expect("couldnt unpack roleid from argument");
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().cloned().unwrap();
    let guild_id=msg.guild_id.expect("got a message from a guild that doesnt exist anymore");
    let channel_id = msg.channel_id;
    args.advance();
    let greeting = args.rest();
    set_greeting_internal(&pool, &guild_id, channel_id, role, greeting.parse().unwrap()).await?;


    Ok(())
}

pub async fn greeting_handler(ctx: Context,old_if_available: Option<Member>, new: Member) -> CommandResult {
    let old = match old_if_available {
        Some(m) => m,
        None => return Ok(()),
    };

    let oldroles = old.roles;
    let newroles = new.roles;

    let role_set: HashSet<_> = oldroles.iter().collect();
    let difference: Vec<_> = newroles.into_iter().filter(|item|!role_set.contains(item)).collect();
    if difference.is_empty() {
        return Ok(())
    }
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().cloned().unwrap();
    let greet_result =get_greeting(&pool, &new.guild_id, *difference.first().unwrap()).await;
    let (channel_id,greeting) = greet_result.expect("couldnt get greeting data from database");
    channel_id.say(ctx.http,greeting).await.expect("couldnt send the greeting. do i have perms?");
      Ok(())
}