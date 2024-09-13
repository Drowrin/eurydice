use crate::{Context, Error, Result};

mod assign;
mod claim;
mod create;
mod delete;
mod edit;
mod release;
mod view;

use poise::Modal;
use serenity::all::{CreateEmbed, CreateEmbedFooter, Member};
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

pub async fn can_manage(ctx: Context<'_>, character: i32) -> Result<()> {
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
        character,
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

pub async fn is_in_game(ctx: Context<'_>, game: i32) -> Result<()> {
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
        game,
        ctx.author().id.get() as i64,
    )
    .fetch_one(&ctx.data().pool)
    .await?
    .exists
    .unwrap();

    if is_player {
        return Ok(());
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
        game,
        ctx.author().id.get() as i64,
    )
    .fetch_one(&ctx.data().pool)
    .await?
    .exists
    .unwrap();

    if is_owner {
        Ok(())
    } else {
        Err(Error::Message("You are not in this game!".to_string()))
    }
}
