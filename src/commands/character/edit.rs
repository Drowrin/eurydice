use poise::{CreateReply, Modal};
use sqlx::query;

use crate::{
    commands::character::{can_manage, character_embed, CharacterModal},
    Context, Result,
};

#[poise::command(slash_command, check = "can_manage")]
pub async fn edit(
    ctx: Context<'_>,
    #[description = "The character to edit"]
    #[autocomplete = "crate::autocomplete::character_editable"]
    character: i32,
) -> Result<()> {
    let maybe_old_character = query!(
        r#"
        select
            name, pronouns, description, image
        from characters
        where id = $1 and guild_id = $2
        "#,
        character,
        ctx.guild_id().unwrap().get() as i64,
    )
    .fetch_optional(&ctx.data().pool)
    .await?;

    let old_character = if let Some(old_character) = maybe_old_character {
        old_character
    } else {
        ctx.say("Character not found! Not sure how you got here...")
            .await?;
        return Ok(());
    };

    let defaults = CharacterModal {
        name: old_character.name,
        pronouns: old_character.pronouns,
        description: old_character.description,
        image: old_character.image,
    };
    let maybe_character_data = CharacterModal::execute_with_defaults(ctx, defaults).await?;

    if let Some(character_data) = maybe_character_data {
        let record = query!(
            r#"
            update characters set
                name = $3,
                pronouns = $4,
                description = $5,
                image = $6
            where id = $1 and guild_id = $2
            returning
                author_id,
                (select title from games where id = game_id) as "game",
                (select user_id from players where character_id = id) as "player"
            "#,
            character,
            ctx.guild_id().unwrap().get() as i64,
            character_data.name.clone(),
            character_data.pronouns.clone(),
            character_data.description.clone(),
            character_data.image.clone(),
        )
        .fetch_one(&ctx.data().pool)
        .await?;

        let player = if let Some(player_id) = record.player {
            Some(
                ctx.guild_id()
                    .unwrap()
                    .member(ctx, player_id as u64)
                    .await?,
            )
        } else {
            None
        };

        ctx.send(
            CreateReply::default().content("Character updated!").embed(
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
