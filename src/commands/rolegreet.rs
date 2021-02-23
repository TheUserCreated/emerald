use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::prelude::*,
    prelude::*,
};
use tracing::{error, info};
use crate::helpers::perms::greetperms;
use crate::structures::data::{ShardManagerContainer, ConnectionPool};
use serenity::static_assertions::_core::str::FromStr;
use crate::db::set_greeting_internal;
use std::collections::HashSet;

#[command]
#[required_permissions("MANAGE_GUILD")]
async fn greeting(ctx: &Context, msg:&Message, mut args: Args) -> CommandResult{
    //let response = greetperms(&ctx,&msg).await;
    //let response = format!("Your perm status was {:?}",response);
    //msg.reply(&ctx,response).await.expect("couldnt respond");
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().cloned().unwrap();

    Ok(())
}


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

pub async fn greeting_handler(ctx: Context,old: Option<Member>, new: Member) -> CommandResult {

    let oldroles = old.expect("cant access old roles").roles;
    let newroles = new.roles;
    let role_set: HashSet<_> = oldroles.iter().collect();
    let difference: Vec<_> = newroles.into_iter().filter(|item|!role_set.contains(item)).collect();
    info!("user with an updated role, role seems to be {:?}",difference);

    Ok(())
}