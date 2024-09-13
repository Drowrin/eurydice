use crate::{Context, Result};

mod create;
mod delete;
mod edit;
mod view;

use poise::Modal;
use serenity::all::CreateEmbed;

#[poise::command(
    slash_command,
    subcommand_required,
    subcommands("create::create", "view::view", "edit::edit", "delete::delete"),
    guild_only
)]
pub async fn system(_: Context<'_>) -> Result<()> {
    Ok(())
}

#[derive(Debug, Default, Modal)]
#[name = "System Details"]
pub struct SystemModal {
    #[name = "Title"]
    #[min_length = 3]
    #[max_length = 100]
    title: String,
    #[name = "Abbreviation"]
    #[min_length = 3]
    #[max_length = 32]
    abbreviation: String,
    #[name = "Description"]
    #[max_length = 1024]
    #[paragraph]
    description: Option<String>,
    #[name = "Image URL"]
    #[max_length = 1024]
    image: Option<String>,
}

type RequiredStringOption = Option<String>;

#[bon::builder]
pub fn system_embed(
    title: String,
    abbreviation: String,
    description: RequiredStringOption,
    image: RequiredStringOption,
) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(title)
        .field("Abbreviation", abbreviation, true);

    if let Some(description) = description {
        embed = embed.field("Description", description, false);
    }

    if let Some(image) = image {
        embed = embed.thumbnail(image);
    }

    embed
}
