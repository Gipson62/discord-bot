use std::sync::Mutex;

use anyhow::Context as _;
use poise::serenity_prelude::{ChannelId, ClientBuilder, EventHandler, GatewayIntents, GuildId, Ready};
use rusqlite::Connection;
use shuttle_runtime::{async_trait, SecretStore};
use shuttle_serenity::ShuttleSerenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with "world!"
#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("world!").await?;
    Ok(())
}

struct Bot {
    database: Mutex<Connection>,
    guild_id: GuildId,
    log_id: ChannelId
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: poise::serenity_prelude::Context, _ready: Ready) {
        let _ = self.log_id.say(&ctx.http, format!("Bot is up")).await;
    }
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![hello()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let bot = Bot {
        database: Mutex::new(
            rusqlite::Connection::open("../database.db").expect("Should be a db there"),
        ),
        guild_id: GuildId::new(
            secret_store
                .get("REDGEAR_GUILD_ID")
                .context("'REDGEAR_GUILD_ID' was not found")?
                .parse::<u64>()
                .expect("Should be parsed correctly"),
        ),
        log_id: ChannelId::new(
            secret_store
                .get("REDGEAR_LOG_ID")
                .context("'REDGEAR_LOG_ID' was not found")?
                .parse::<u64>()
                .expect("Should be parsed correctly"),
        ),
    };

    let client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .event_handler(bot)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
