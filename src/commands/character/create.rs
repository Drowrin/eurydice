use poise::{CreateReply, Modal};
use sqlx::query;

use crate::{
    commands::character::{character_embed, is_in_game, CharacterModal},
    Context, Result,
};

#[poise::command(slash_command, check = "is_in_game")]
pub async fn create(
    ctx: Context<'_>,
    #[description = "The name of the character"] name: String,
    #[description = "The game to add the character to"]
    #[autocomplete = "crate::autocomplete::game_joined"]
    game: i32,
) -> Result<()> {
    let already_exists = query!(
        r#"
        select exists(
            select 1
            from characters
            where
                game_id = $2
                and
                name = $1
        )
        "#,
        name,
        game,
    )
    .fetch_one(&ctx.data().pool)
    .await?
    .exists
    .unwrap();

    if already_exists {
        ctx.reply(format!("`{name}` already exists!")).await?;
        return Ok(());
    }

    let defaults = CharacterModal {
        name,
        ..Default::default()
    };
    let maybe_character_data = CharacterModal::execute_with_defaults(ctx, defaults).await?;

    if let Some(character_data) = maybe_character_data {
        let auto_assign = query!(
            r#"
            select exists (
                select 1
                from players
                where
                    game_id = $1
                    and
                    user_id = $2
                    and
                    character_id is null
            )
            "#,
            game,
            ctx.author().id.get() as i64,
        )
        .fetch_one(&ctx.data().pool)
        .await?
        .exists
        .unwrap();

        let record = query!(
            r#"
            insert
            into characters
                (
                    guild_id, author_id, game_id,
                    name, pronouns, description, image
                )
            values
                ($1, $2, $3, $4, $5, $6, $7)
            returning
                id,
                (select title from games where id = $3) as "game"
            "#,
            ctx.guild_id().unwrap().get() as i64,
            ctx.author().id.get() as i64,
            game,
            character_data.name.clone(),
            character_data.pronouns.clone(),
            character_data.description.clone(),
            character_data.image.clone(),
        )
        .fetch_one(&ctx.data().pool)
        .await?;

        let player = if auto_assign {
            query!(
                r#"
                update players
                set
                    character_id = $3
                where
                    game_id = $1
                    and
                    user_id = $2
                "#,
                game,
                ctx.author().id.get() as i64,
                record.id,
            )
            .execute(&ctx.data().pool)
            .await?;
            Some(ctx.author_member().await.unwrap().into_owned())
        } else {
            None
        };

        let content = if auto_assign {
            "Character created!\nIt was auto-assigned to you!"
        } else {
            "Character created!"
        };
        ctx.send(
            CreateReply::default().content(content).embed(
                character_embed()
                    .name(character_data.name)
                    .pronouns(character_data.pronouns)
                    .description(character_data.description)
                    .image(character_data.image)
                    .game(record.game.unwrap())
                    .player(player)
                    .call(),
            ),
        )
        .await?;
    }

    Ok(())
}
