use crate::{Context, Data, Error, Result};

mod activate;
mod create;
mod deactivate;
mod delete;
mod edit;
mod transfer;
mod view;

mod player;
mod system;

use eyre::eyre;
use poise::Modal;
use serenity::all::{
    ChannelId, CreateEmbed, CreateEmbedFooter, Mentionable, ResolvedValue, RoleId, UserId,
};
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

pub async fn can_manage(ctx: poise::Context<'_, Data, Error>) -> Result<bool> {
    let game_id = match ctx {
        poise::Context::Application(c) => {
            let resolved_value = c
                .args
                .iter()
                .find(|a| a.name == "game")
                .ok_or(eyre!(
                    "Argument 'game' not found when game::can_manage check was used"
                ))?
                .value
                .clone();
            match resolved_value {
                ResolvedValue::Integer(i) => i as i32,
                ResolvedValue::Autocomplete { .. } => return Ok(true),
                _ => {
                    return Err(eyre!(
                        "Argument 'game' was not an integer when game::can_manage check was used"
                    )
                    .into());
                }
            }
        }
        poise::Context::Prefix(c) => c
            .args
            .split_whitespace()
            .next()
            .ok_or(eyre!("No args when game::can_manage check was used"))?
            .parse::<i32>()
            .map_err(|_| eyre!("Could not parse arg in check game::can_manage"))?,
    };

    if ctx
        .author_member()
        .await
        .unwrap()
        .permissions
        .unwrap()
        .manage_messages()
    {
        return Ok(true);
    }

    let record = query!(
        r#"
        select exists (
            select 1
            from games
            where id = $1 and owner_id = $2
        )
        "#,
        game_id,
        ctx.author().id.get() as i64
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    Ok(record.exists.unwrap())
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
