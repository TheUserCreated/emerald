use color_thief::ColorFormat;

use serenity::cache::FromStrAndCache;
use serenity::utils::Colour;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
async fn getcolour(ctx: &Context, user_id: UserId) -> CommandResult<(u8, u8, u8)> {
    let user = {
        let user = ctx
            .http
            .get_user(u64::from(user_id))
            .await
            .expect("couldn't get a user");
        user
    };
    let img = {
        let img = user.avatar_url().expect("couldn't get users' avatar");
        let img = img.replace(".webp?size=1024", ".png?size=1024");
        let response = reqwest::get(img)
            .await
            .expect("couldn't get response in http req");
        let img_bytes = response
            .bytes()
            .await
            .expect("couldn't get response as bytes");
        image::load_from_memory(&*img_bytes)
            .expect("couldn't get image")
            .to_bgr8()
    };
    let colour_buffer = color_thief::get_palette(img.as_raw(), ColorFormat::Bgr, 10, 3)
        .expect("couldn't make palette");
    Ok((colour_buffer[0].r, colour_buffer[0].g, colour_buffer[0].b))
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_ROLES")]
async fn rolecoloursync(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let id = {
        let id = args.current().expect("not enough args");
        UserId::from_str(ctx, id).await.expect("no such user?")
    };
    args.advance();
    let role = {
        let id = args.current().expect("not enough args");
        Role::from_str(ctx, id).await.expect("couldn't get role")
    };
    let guild = ctx
        .http
        .get_guild(u64::from(
            msg.guild_id.expect("message from guild that doesnt exist"),
        ))
        .await
        .expect("no guild");
    let roles = guild.roles;
    for guild_role in roles.into_values() {
        if !(msg.author.id
            == msg
                .guild(ctx)
                .await
                .expect("cant access that guild")
                .owner_id)
            && guild_role.position >= role.position
        {
            msg.reply(
                ctx,
                "You can't edit your highest role or roles higher than it.",
            )
            .await?;
        }
    }
    let colour = getcolour(ctx, UserId::from(id.0))
        .await
        .expect("couldnt get colour");
    let colour = Colour::from_rgb(colour.0, colour.1, colour.2);
    role.edit(ctx, |r| {
        r.colour(colour.0 as u64);
        r
    })
    .await?;
    Ok(())
}
