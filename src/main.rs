use std::sync::Mutex;

use anyhow::Context as AnyhowContext;
use poise::serenity_prelude::{ChannelId, ClientBuilder, GatewayIntents, GuildId, User};
use rusqlite::Connection;
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

mod events;

pub struct Data {
    pub conn: Mutex<Connection>,
    pub guild_id: GuildId,
    pub log_id: ChannelId,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with "world!"
#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("world!").await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn test(
    ctx: Context<'_>,
    #[description = "User to be pinged"] user_to_ping: User,
) -> Result<(), Error> {
    ctx.data()
        .log_id
        .say(
            ctx,
            format!("{} pinged: <@{}>", ctx.author(), user_to_ping.id),
        )
        .await?;
    Ok(())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let guild_id = secret_store
        .get("REDGEAR_GUILD_ID")
        .context("'REDGEAR_GUILD_ID' was not found")?
        .parse::<u64>()
        .expect("Should be parsed correctly");

    let log_id = secret_store
        .get("REDGEAR_LOG_ID")
        .context("'REDGEAR_LOG_ID' was not found")?
        .parse::<u64>()
        .expect("Should be parsed correctly");

    let options = poise::FrameworkOptions {
        commands: vec![hello(), test()],
        event_handler: |ctx, event, framework, data| {
            Box::pin(events::handle(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .options(options)
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                for guild in &ready.guilds {
                    if guild.unavailable {
                        let _ = poise::builtins::register_in_guild(
                            ctx,
                            &framework.options().commands,
                            guild.id,
                        )
                        .await;
                    }
                }
                Ok(Data {
                    conn: Mutex::new(
                        rusqlite::Connection::open("../database.db").expect("Should be a db there"),
                    ),
                    guild_id: GuildId::new(guild_id),
                    log_id: ChannelId::new(log_id),
                })
            })
        })
        .build();

    let mut client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .expect("Failed to create a client");

    let _ = client.start().await;

    Ok(client.into())
}
