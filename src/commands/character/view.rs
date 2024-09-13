use poise::CreateReply;
use sqlx::query;

use crate::{commands::character::character_embed, Context, Result};

#[poise::command(slash_command)]
pub async fn view(
    ctx: Context<'_>,
    #[description = "The character to view"]
    #[autocomplete = "crate::autocomplete::character"]
    character: i32,
) -> Result<()> {
    let maybe_character = query!(
        r#"
        select
            name, pronouns, description, image, author_id,
            (select title from games where id = game_id) as "game",
            (select user_id from players where character_id = $1) as "player"
        from characters
        where id = $1 and guild_id = $2
        "#,
        character,
        ctx.guild_id().unwrap().get() as i64,
    )
    .fetch_optional(&ctx.data().pool)
    .await?;

    match maybe_character {
        Some(character) => {
            let player = if let Some(player) = character.player {
                Some(ctx.guild_id().unwrap().member(ctx, player as u64).await?)
            } else {
                None
            };
            ctx.send(
                CreateReply::default().embed(
                    character_embed()
                        .name(character.name)
                        .pronouns(character.pronouns)
                        .description(character.description)
                        .image(character.image)
                        .game(character.game.unwrap())
                        .player(player)
                        .call(),
                ),
            )
            .await?;
        }
        None => {
            ctx.say("System not found! Not sure how you got here...")
                .await?;
        }
    }

    Ok(())
}
