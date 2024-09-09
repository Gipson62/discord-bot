use poise::{
    serenity_prelude::{self as serenity, ActivityData, FullEvent, Ready},
    FrameworkContext,
};

use crate::{Data, Error};

pub async fn handle(
    ctx: &serenity::prelude::Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot } => on_ready(ctx, data_about_bot, framework).await,
        _ => (),
    }
    Ok(())
}

async fn on_ready(
    ctx: &serenity::Context,
    _ready: &Ready,
    framework: FrameworkContext<'_, Data, Error>,
) {
    ctx.set_activity(Some(ActivityData::playing("something")));
    let _ = framework.user_data.log_id.say(ctx, "Hello I'm on").await;
}
