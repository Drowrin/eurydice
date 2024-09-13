use sqlx::query;

use crate::{commands::character::can_manage, Context, Result};

#[poise::command(slash_command, check = "can_manage")]
pub async fn release(
    ctx: Context<'_>,
    #[description = "The character to release"]
    #[autocomplete = "crate::autocomplete::character_assigned"]
    character: i32,
) -> Result<()> {
    let record = query!(
        r#"
        update players
        set character_id = null
        where character_id = $1
        returning (select name from characters where id = $1)
        "#,
        character
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    ctx.say(format!("`{}` released.", record.name.unwrap()))
        .await?;

    Ok(())
}
