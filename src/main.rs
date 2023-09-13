use anyhow::Context as _;
use poise::{serenity_prelude as serenity, PartialContext};
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use std::sync::Mutex;

struct Data {
    prefix: Mutex<Option<String>>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn dynamic_prefix(
    ctx: poise::PartialContext<'_, Data, Error>,
) -> Result<Option<String>, Error> {
    // Here you can fetch the prefix dynamically, for example from a database.
    // For simplicity, we return a static prefix here.
    Ok(Some("my_prefix".to_string()))
}

/// Responds with "world!"
#[poise::command(slash_command, prefix_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("world!").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn wood(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("world!").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn prefix(ctx: Context<'_>, prefix: String) -> Result<(), Error> {
    let mut data = ctx.data().prefix.lock().expect("Can't lock!?");
    *data = Some(prefix);
    Ok(())
}

#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![hello(), wood(), prefix()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: None,
                dynamic_prefix: Some(Box::pin(async move { Ok(Some("my_prefix".to_string())) })),
                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                )),
                case_insensitive_commands: true,
                mention_as_prefix: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .token(discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { prefix })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
