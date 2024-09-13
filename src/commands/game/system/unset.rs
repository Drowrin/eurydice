use sqlx::query;

use crate::{commands::game::can_manage, Context, Result};

#[poise::command(slash_command, check = "can_manage", ephemeral)]
pub async fn unset(
    ctx: Context<'_>,
    #[description = "The game to unset the system of"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
) -> Result<()> {
    let record = query!(
        r#"
        update games
        set system_id = null
        where guild_id = $1 and id = $2
        returning title
        "#,
        ctx.guild_id().unwrap().get() as i64,
        game,
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    ctx.say(format!("Unset system of `{}`.", record.title))
        .await?;

    Ok(())
}
