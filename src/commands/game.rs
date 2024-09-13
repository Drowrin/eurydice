use crate::{Context, Error, Result};

mod activate;
mod create;
mod deactivate;
mod delete;
mod edit;
mod transfer;
mod view;

mod player;
mod system;

use poise::Modal;
use serenity::all::{ChannelId, CreateEmbed, CreateEmbedFooter, Mentionable, RoleId, UserId};
use sqlx::{
    query,
    types::chrono::{DateTime, Utc},
};

#[poise::command(
    slash_command,
    subcommand_required,
    subcommands(
        "player::player",
        "system::system",
        "create::create",
        "view::view",
        "edit::edit",
        "delete::delete",
        "transfer::transfer",
        "activate::activate",
        "deactivate::deactivate",
    ),
    guild_only
)]
pub async fn game(_: Context<'_>) -> Result<()> {
    Ok(())
}

pub async fn can_manage(ctx: Context<'_>, game: i32) -> Result<()> {
    if ctx
        .author_member()
        .await
        .unwrap()
        .permissions
        .unwrap()
        .manage_messages()
    {
        return Ok(());
    }

    let record = query!(
        r#"
        select exists (
            select 1
            from games
            where id = $1 and owner_id = $2
        )
        "#,
        game,
        ctx.author().id.get() as i64
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    if record.exists.unwrap() {
        Ok(())
    } else {
        Err(Error::Message(
            "You don't have permission to do that!".to_string(),
        ))
    }
}

#[derive(Debug, Default, Modal)]
#[name = "Game Details"]
pub struct GameModal {
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
type RequiredChannelOption = Option<ChannelId>;

#[bon::builder]
pub fn game_embed(
    title: String,
    abbreviation: String,
    description: RequiredStringOption,
    image: RequiredStringOption,
    system: RequiredStringOption,
    created_at: DateTime<Utc>,
    role_id: RoleId,
    channel_id: RequiredChannelOption,
    owner_id: UserId,
    players: Vec<UserId>,
) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("[{}] {}", abbreviation, title))
        .footer(CreateEmbedFooter::new("Created"))
        .timestamp(created_at)
        .field("Role", role_id.mention().to_string(), true);

    if let Some(channel_id) = channel_id {
        embed = embed.field("Main Channel", channel_id.mention().to_string(), true);
    }

    if let Some(system_abbreviation) = system {
        embed = embed.field("System", system_abbreviation, true);
    }

    embed = embed.field(
        "Players",
        if players.is_empty() {
            owner_id.mention().to_string()
        } else {
            format!(
                "{} | {}",
                owner_id.mention(),
                players
                    .into_iter()
                    .map(|p| p.mention().to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            )
        },
        false,
    );

    if let Some(description) = description {
        embed = embed.field("Description", description, false);
    }

    if let Some(image) = image {
        embed = embed.thumbnail(image);
    }

    embed
}
