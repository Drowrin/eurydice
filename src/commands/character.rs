use crate::{Context, Data, Error, Result};

mod assign;
mod claim;
mod create;
mod delete;
mod edit;
mod release;
mod view;

use eyre::eyre;
use poise::Modal;
use serenity::all::{CreateEmbed, CreateEmbedFooter, Member, ResolvedValue};
use sqlx::query;

#[poise::command(
    slash_command,
    subcommand_required,
    subcommands(
        "create::create",
        "view::view",
        "edit::edit",
        "delete::delete",
        "claim::claim",
        "release::release",
        "assign::assign",
    ),
    guild_only
)]
pub async fn character(_: Context<'_>) -> Result<()> {
    Ok(())
}

#[derive(Debug, Default, Modal)]
#[name = "Game Details"]
pub struct CharacterModal {
    #[name = "Name"]
    #[min_length = 3]
    #[max_length = 32]
    name: String,
    #[name = "Pronouns"]
    #[max_length = 20]
    pronouns: Option<String>,
    #[name = "Description"]
    #[max_length = 1024]
    #[paragraph]
    description: Option<String>,
    #[name = "Image URL"]
    #[max_length = 1024]
    image: Option<String>,
}

type RequiredStringOption = Option<String>;
type RequiredMemberOption = Option<Member>;

#[bon::builder]
pub fn character_embed(
    name: String,
    pronouns: RequiredStringOption,
    description: RequiredStringOption,
    image: RequiredStringOption,
    game: String,
    player: RequiredMemberOption,
) -> CreateEmbed {
    let mut embed = CreateEmbed::new().title(name);

    if let Some(pronouns) = pronouns {
        embed = embed.field("Pronouns", pronouns, true);
    }

    embed = embed.field("Game", game, true);

    if let Some(description) = description {
        embed = embed.field("Description", description, false);
    }

    if let Some(image) = image {
        embed = embed.image(image);
    }

    if let Some(player) = player {
        let mut footer = CreateEmbedFooter::new(format!("played by {}", player.display_name()));
        if let Some(avatar_url) = player.avatar_url().or(player.user.avatar_url()) {
            footer = footer.icon_url(avatar_url);
        }
        embed = embed.footer(footer);
    }

    embed
}

pub async fn can_manage(ctx: poise::Context<'_, Data, Error>) -> Result<bool> {
    let character_id = match ctx {
        poise::Context::Application(c) => {
            let resolved_value = c
                .args
                .iter()
                .find(|a| a.name == "character")
                .ok_or(eyre!(
                    "Argument 'character' not found when character::can_manage check was used"
                ))?
                .value
                .clone();
            match resolved_value {
                ResolvedValue::Integer(i) => i as i32,
                ResolvedValue::Autocomplete { .. } => return Ok(true),
                _ => {
                    return Err(eyre!(
                        "Argument 'character' was not an integer when character::can_manage check was used"
                    )
                    .into());
                }
            }
        }
        poise::Context::Prefix(c) => c
            .args
            .split_whitespace()
            .next()
            .ok_or(eyre!("No args when character::can_manage check was used"))?
            .parse::<i32>()
            .map_err(|_| eyre!("Could not parse arg in check character::can_manage"))?,
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
            from characters as c
            where
                id = $1
                and
                (
                    exists (
                        select 1
                        from players
                        where
                            character_id = $1
                            and
                            user_id = $2
                    )
                    or
                    exists (
                        select 1
                        from games
                        where
                            id = c.id
                            and
                            owner_id = $2
                    )
                )
        )
        "#,
        character_id,
        ctx.author().id.get() as i64
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    Ok(record.exists.unwrap())
}

pub async fn is_in_game(ctx: poise::Context<'_, Data, Error>) -> Result<bool> {
    let game_id = match ctx {
        poise::Context::Application(c) => match c.args.iter().find(|a| a.name == "game") {
            Some(game_arg) => match game_arg.value.clone() {
                ResolvedValue::Integer(i) => i as i32,
                ResolvedValue::Autocomplete { .. } => return Ok(true),
                _ => {
                    return Err(eyre!("Argument 'game' was not an integer when character::can_manage check was used").into());
                }
            },
            None => {
                let resolved_value = c
                    .args
                    .iter()
                    .find(|a| a.name == "character")
                    .ok_or(eyre!(
                        "Argument 'character' not found when character::can_manage check was used"
                    ))?
                    .value
                    .clone();
                match resolved_value {
                    ResolvedValue::Integer(character_id) => {
                        let character_id = character_id as i32;
                        query!(
                            r#"
                            select game_id
                            from characters
                            where id = $1
                            "#,
                            character_id
                        )
                        .fetch_one(&ctx.data().pool)
                        .await?
                        .game_id
                    }
                    ResolvedValue::Autocomplete { .. } => return Ok(true),
                    _ => {
                        return Err(eyre!("Argument 'character' was not an integer when character::can_manage check was used").into());
                    }
                }
            }
        },
        poise::Context::Prefix(_) => {
            return Err(eyre!("Somehow got a prefix context in character::is_in_game").into())
        }
    };

    let is_player = query!(
        r#"
        select exists (
            select 1
            from players
            where
                game_id = $1
                and
                user_id = $2
        )
        "#,
        game_id,
        ctx.author().id.get() as i64,
    )
    .fetch_one(&ctx.data().pool)
    .await?
    .exists
    .unwrap();

    if is_player {
        return Ok(true);
    }

    let is_owner = query!(
        r#"
        select exists (
            select 1
            from games
            where
                id = $1
                and
                owner_id = $2
        )
        "#,
        game_id,
        ctx.author().id.get() as i64,
    )
    .fetch_one(&ctx.data().pool)
    .await?
    .exists
    .unwrap();

    Ok(is_owner)
}
