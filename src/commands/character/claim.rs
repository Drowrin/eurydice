use serenity::all::Mentionable;
use sqlx::query;

use crate::{
    commands::{character::is_in_game, contextual_args},
    Context, Result,
};

/// Claim a character that currently has no player. Usable by all players.
#[poise::command(slash_command)]
pub async fn claim(
    ctx: Context<'_>,
    #[description = "The character to claim"]
    #[autocomplete = "crate::autocomplete::character_claimable"]
    character: i32,
) -> Result<()> {
    let ctx_args = contextual_args()
        .character_id_arg(Some(character))
        .ctx(&ctx)
        .call()
        .await?;

    is_in_game(ctx, ctx_args.game_id).await?;

    let record = query!(
        r#"
        update players
        set character_id = $1
        where
            user_id = $2
            and
            game_id = (
                select game_id
                from characters
                where id = $1
            )
        returning (select name from characters where id = $1)
        "#,
        character,
        ctx.author().id.get() as i64,
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    ctx.say(format!(
        "`{}` claimed by {}.",
        record.name.unwrap(),
        ctx.author_member().await.unwrap().mention()
    ))
    .await?;

    Ok(())
}
