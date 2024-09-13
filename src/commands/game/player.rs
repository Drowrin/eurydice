use crate::{Context, Result};

mod add;
mod remove;

#[poise::command(
    slash_command,
    subcommand_required,
    subcommands("add::add", "remove::remove"),
    guild_only
)]
pub async fn player(_: Context<'_>) -> Result<()> {
    Ok(())
}
