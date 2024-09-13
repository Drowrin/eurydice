use dotenv::dotenv;
use eyre::{Context, Result};
use poise::{builtins::register_globally, samples::register_in_guild};
use serenity::all::{GuildId, Http};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let token = env::var("DISCORD_TOKEN")?;
    let http = Http::new(&token);

    let app_id = http.get_current_application_info().await?.id;
    let install_url = format!("https://discord.com/oauth2/authorize?client_id={app_id}");

    http.set_application_id(app_id);

    let commands = eurydice::commands::all();

    match env::var("TEST_GUILD") {
        Ok(id) => {
            let guild_id = GuildId::from(id.parse::<u64>()?);
            let guild = http.get_guild(guild_id).await.context(format!(
                "Couldn't find guild ({}), did you invite the bot? {}",
                guild_id, install_url
            ))?;
            println!("Publishing commands to {} ({})", guild.name, guild_id);
            register_in_guild(http, commands.as_slice(), guild_id).await?;
        }
        Err(_) => {
            println!("Publishing commands globally");
            register_globally(http, commands.as_slice()).await?;
        }
    }

    println!("Success");

    Ok(())
}
