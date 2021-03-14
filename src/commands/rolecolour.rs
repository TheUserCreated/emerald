use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::prelude::*,
    prelude::*,
};
use tracing::{info};
use serenity::cache::FromStrAndCache;
use color_thief::ColorFormat;


#[command]
async fn getcolour (ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let user = {
         let id = args.current().expect("not enough args");
    let id = UserId::from_str(ctx,id).await.expect("no such user?");
    let user = ctx.http.get_user(u64::from(id)).await.expect("couldn't get a user");
        user
    };
    let img = {
        let img = user.avatar_url().expect("couldn't get users' avatar");
        let response = reqwest::get(img).await.expect("couldn't get response in http req");
        let img_bytes = response.bytes().await.expect("couldn't get response as bytes");
        image::load_from_memory(&*img_bytes).expect("couldn't get image").to_rgba8()

    };

    /*let mut pix1: u64 = 0;
    let mut pix2: u64 = 0;
    let mut pix3: u64 = 0;
    let mut len: u64 = 0;
    for pix in img.pixels() {
        pix1 += pix[0] as u64;
        pix2 += pix[1] as u64 ;
        pix3 += pix[2] as u64;
        len +=1;
    };
    let pix1 = pix1 / len;
    let pix2 = pix2 / len;
    let pix3 = pix3 / len;
    let response = format!("colour is {} {} {}", pix1,pix2,pix3); */
    let colour_buffer = color_thief::get_palette(img.as_raw(),ColorFormat::Rgb,10,3).expect("couldnt make palette");
    let response = format!("Your palette is {:?}",colour_buffer);
    msg.reply(&ctx.http, response).await?;


    Ok(())
}
