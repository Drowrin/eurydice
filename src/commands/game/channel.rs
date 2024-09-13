use crate::{Context, Result};

mod set;
mod unset;

#[poise::command(
    slash_command,
    subcommand_required,
    subcommands("set::set", "unset::unset"),
    guild_only
)]
pub async fn channel(_: Context<'_>) -> Result<()> {
    Ok(())
}
