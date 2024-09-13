use serenity::all::{Member, Mentionable};
use sqlx::query;

use crate::{commands::character::can_manage, Context, Result};

#[poise::command(slash_command)]
pub async fn assign(
    ctx: Context<'_>,
    #[description = "The character to assign"]
    #[autocomplete = "crate::autocomplete::character_assignable"]
    character: i32,
    #[description = "The user to assign the character to"] user: Member,
) -> Result<()> {
    can_manage(ctx, character).await?;

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
        user.user.id.get() as i64,
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    ctx.say(format!(
        "`{}` assigned to {}.",
        record.name.unwrap(),
        user.mention()
    ))
    .await?;

    Ok(())
}
