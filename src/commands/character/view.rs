use poise::CreateReply;
use sqlx::query;

use crate::{
    commands::{character::character_embed, contextual_args},
    Context, Result,
};

/// View a character's details. Usable by anyone.
#[poise::command(slash_command)]
pub async fn view(
    ctx: Context<'_>,
    #[description = "The character to view"]
    #[autocomplete = "crate::autocomplete::character"]
    character: Option<i32>,
) -> Result<()> {
    let character = contextual_args()
        .character_id_arg(character)
        .ctx(&ctx)
        .call()
        .await?
        .character_id
        .unwrap();

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
            let player = match character.player {
                Some(player) => Some(ctx.guild_id().unwrap().member(ctx, player as u64).await?),
                _ => None,
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
