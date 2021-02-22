use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::structures::data::ShardManagerContainer;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;
            return Ok(());
        }
    };
    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    //shards are backed by runners that are responsible for the shards events
    //so to get the latency we gotta get some info from the current runner
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found. How did that even happen?")
                .await?;
            return Ok(());
        }
    };
    let latency = match runner.latency {
        Some(latency) => latency,
        None => {
            msg.reply(ctx, "This shard hasn't been active long enough to measure its own latency. Please wait.").await?;
            return Ok(());
        }
    };
    msg.reply(ctx, &format!("The shard latency is {:?}", latency))
        .await?;

    Ok(())
}

#[command]
#[owners_only]
async fn die(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.reply(ctx, "Shutting down!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager")
            .await?;

        return Ok(());
    }

    Ok(())
}