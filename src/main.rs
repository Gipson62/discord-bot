use anyhow::Context as _;
use poise::{serenity_prelude::{
    ClientBuilder, CommandOptionType, CreateCommand, CreateCommandOption, EventHandler,
    GatewayIntents, Guild, GuildId, Ready,
}, SlashArgument};
use shuttle_runtime::{async_trait, SecretStore};
use shuttle_serenity::ShuttleSerenity;
use sqlite::{open, ConnectionThreadSafe};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with "world!"
#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("world!").await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn test(ctx: Context<'_>, ) -> Result<(), Error> {
    Ok(())
}

struct Bot {
    database: ConnectionThreadSafe,
    guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: poise::serenity_prelude::Context, ready: Ready) {
        let _ = self.guild_id
            .set_commands(
                &ctx.http,
                vec![CreateCommand::new("todo")
                    .description("Add, list and complete todos")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "add",
                            "Add a new todo",
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::String,
                                "note",
                                "The todo note to add",
                            )
                            .min_length(2)
                            .max_length(100)
                            .required(true),
                        ),
                    )
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "complete",
                            "The todo to complete",
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::Integer,
                                "index",
                                "The index of the todo to complete",
                            )
                            .min_int_value(1)
                            .required(true),
                        ),
                    )
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "list",
                        "List your todos",
                    ))],
            )
            .await;
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
        database: sqlite::Connection::open_thread_safe("../database.sqlite")
            .expect("should be the correct path but not sure"),
        guild_id: GuildId::new(
            secret_store
                .get("REDGEAR_GUILD_ID")
                .context("'REDGEAR_GUILD_ID' was not found")?
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
