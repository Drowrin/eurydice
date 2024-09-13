use serenity::all::Mentionable;
use sqlx::query;

use crate::{commands::character::is_in_game, Context, Result};

#[poise::command(slash_command, check = "is_in_game")]
pub async fn claim(
    ctx: Context<'_>,
    #[description = "The character to claim"]
    #[autocomplete = "crate::autocomplete::character_assignable"]
    character: i32,
) -> Result<()> {
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
