use color_thief::ColorFormat;
use serenity::cache::FromStrAndCache;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use tracing::info;

#[command]
async fn getcolour(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    //info!("Here be dragons")
    let user = {
        let id = args.current().expect("not enough args");
        let id = UserId::from_str(ctx, id).await.expect("no such user?");
        let user = ctx
            .http
            .get_user(u64::from(id))
            .await
            .expect("couldn't get a user");
        user
    };
    let img = {
        let img = user.avatar_url().expect("couldn't get users' avatar");
        let response = reqwest::get(img)
            .await
            .expect("couldn't get response in http req");
        let img_bytes = response
            .bytes()
            .await
            .expect("couldn't get response as bytes");
        image::load_from_memory(&*img_bytes)
            .expect("couldn't get image")
            .to_rgb8()
    }; //TODO once the image crate supports webP this needs adjusting.
    let colour_buffer = color_thief::get_palette(img.as_raw(), ColorFormat::Rgb, 10, 3)
        .expect("couldnt make palette");
    let response = format!("Your palette is {:?}", colour_buffer[0]);
    msg.reply(&ctx.http, response).await?;

    Ok(())
}
