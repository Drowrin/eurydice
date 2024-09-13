use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv()?;

    let token = env::var("DISCORD_TOKEN")?;
    let intents = serenity::GatewayIntents::non_privileged();

    let db_url = env::var("DATABASE_URL")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: eurydice::commands::all(),
            on_error: |error| Box::pin(eurydice::error::handle(error)),
            ..Default::default()
        })
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(eurydice::Data {
                    pool: PgPoolOptions::new()
                        .max_connections(5)
                        .connect(&db_url)
                        .await?,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    Ok(client.start().await?)
}
