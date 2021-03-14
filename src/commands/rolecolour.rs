use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::prelude::*,
    prelude::*,
};
use tracing::{info};
use serenity::cache::FromStrAndCache;
use image::{ImageFormat, EncodableLayout};
use color_thief;
use tokio::fs::File;
use color_thief::ColorFormat;

#[command]
async fn getcolour (ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    info!("running command");
    let user = {
         let id = args.current().expect("not enough args");
    let id = UserId::from_str(ctx,id).await.expect("no such user?");
    let user = ctx.http.get_user(u64::from(id)).await.expect("couldn't get a user");
        user
    };
    info!("got user, getting image");
    let (img, mut img_bytes) = {
        let img = user.avatar_url().expect("couldn't get users' avatar");
        let response = reqwest::get(img).await.expect("couldn't get response in http req");
        let img_bytes = response.bytes().await.expect("couldn't get response as bytes");
        (image::load_from_memory(&*img_bytes).expect("couldn't get image"),
         img_bytes,)

    };
    info!("got image, saving");
    //img.save_with_format(user.id.as_u64().to_string(), ImageFormat::Png)?;
    info!("saved image");
    //let path = format!("attachment://{}",user.id.as_u64().to_string());
    //let img_file = File::open(&path).await?;7878
    //tokio::fs::remove_file(user.id.as_u64().to_string()).await.expect("couldn't remove file");
    let mut bytes_vec = img_bytes.to_vec();
    let colour_buffer = color_thief::get_palette(&*img_bytes, ColorFormat::Rgb, 10, 2);
    info!("{:?}",colour_buffer);


    Ok(())
}
