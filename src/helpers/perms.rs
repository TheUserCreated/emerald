use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::atomic::AtomicBool;

pub async fn greetperms(ctx: &Context,msg : &Message) -> AtomicBool {
    let guild = msg.guild(&ctx).await.expect("got a message from a guild that doesnt exist?");
    let perms = match guild
        .member_permissions(&ctx,msg.author.id)
        .await
    {
        Ok(perms) => perms,
        Err(_) => return AtomicBool::new(false),
    };

    println!("{:?}",perms);
    AtomicBool::new(false)
}