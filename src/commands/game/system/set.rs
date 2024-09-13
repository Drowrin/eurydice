use sqlx::query;

use crate::{commands::game::can_manage, Context, Result};

#[poise::command(slash_command, check = "can_manage", ephemeral)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "The game to set the system of"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
    #[description = "The system to set"]
    #[autocomplete = "crate::autocomplete::system"]
    system: i32,
) -> Result<()> {
    let record = query!(
        r#"
        update games
        set system_id = $3
        where guild_id = $1 and id = $2
        returning title, (select title from systems where id = $3) as "system"
        "#,
        ctx.guild_id().unwrap().get() as i64,
        game,
        system,
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    ctx.say(format!(
        "Set system of `{}` to `{}`.",
        record.title,
        record.system.unwrap()
    ))
    .await?;

    Ok(())
}
