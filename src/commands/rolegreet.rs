use std::collections::HashSet;
use ordinal::Ordinal;
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
#[sub_commands(remove, add, replacements)]
#[description = "This command allows you to send users a message in a channel of your choosing whenever a user receives the specified role. \n \
Certain pieces of text are replaced, to see which do `greeting replacements`"]
#[only_in(guilds)]
async fn greeting(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    msg.reply(ctx, "Use add, remove, or list.").await?;

    Ok(())
}

#[command]
#[required_permissions("MANAGE_GUILD")]
#[description = "Allows you to add greetings, to be sent in the channel where the command is executed."]
#[usage = "add @helper Hi %usermention%, you received the helper role!"]
#[only_in(guilds)]
#[aliases("set")]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
#[aliases("delete")]
#[description = "Allows you to remove greetings from the channel the command is executed"]
#[usage = "remove @helper"]
#[only_in(guilds)]
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

#[command]
#[description = "Lists all text that is replaced in greetings"]
#[only_in(guilds)]
async fn replacements(ctx: &Context, msg: &Message) ->CommandResult{

    msg.reply(ctx, "The text %servername% is replaced with your servers name.\n\
    %usermention% is replaced with a mention directed at the user being greeted. \n\
    %channelmention% is replaced with a mention of the channel the greeting is sent in. \n\
    %roleid% is replaced with the role ID for the role the user has just received.").await?;

    Ok(())

}

pub async fn greeting_handler(ctx: Context, old_if_available: Option<Member>, new: Member) -> CommandResult {
    let old = match old_if_available {
        Some(m) => m,
        None => return Ok(()),
    };
    let difference =
        {
            let oldroles = &old.roles;
            let newroles = &new.roles;
            let role_set: HashSet<_> = oldroles.iter().collect();
            let difference: Vec<_> = newroles.iter().filter(|item| !role_set.contains(item)).collect();
            if difference.is_empty() {
                return Ok(());
            } else {
                difference
            }
        };

    let role_id = difference.first().unwrap();
    let role_id = role_id.to_owned().to_owned();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().cloned().unwrap();
    let greet_result = get_greeting(&pool, &new.guild_id, &role_id).await;
    let (channel_id, greeting) = greet_result.expect("couldn't get greeting data from database");
    let greeting = greeting_replacements(&ctx, &new, greeting, &channel_id, &role_id).await.expect("failure at greeting text replacement!");
    if channel_id.0 == 0 {
        return Ok(());
    }
    debug!("greeting attempting to be sent in channel {:?}", channel_id);
    channel_id.say(ctx.http, greeting).await.expect("couldn't send the greeting. do i have perms?");
    Ok(())
}

pub async fn greeting_replacements(ctx: &Context, new: &Member, text: String, channel_id: &ChannelId, role_id: &RoleId) -> CommandResult<String> {
    let guild_name = new.guild_id.name(&ctx).await.expect("couldn't get guild name to replace");
    let finished_text = text.replace("%servername%", &*guild_name);
    let finished_text = finished_text.replace("%usermention%", new.mention().to_string().as_str());
    let finished_text = finished_text.replace("%channelmention%", channel_id.mention().to_string().as_str());
    let finished_text = finished_text.replace("%roleid%", role_id.to_string().as_str());
    Ok(finished_text)
}