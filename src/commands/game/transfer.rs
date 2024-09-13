use serenity::all::{Mentionable, User};
use sqlx::query;

use crate::{commands::game::can_manage, Context, Result};

#[poise::command(slash_command)]
pub async fn transfer(
    ctx: Context<'_>,
    #[description = "The game to transfer"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
    #[description = "The user to transfer ownership of the game to"] user: User,
    #[description = "Cause owner to also leave the game as a player"] also_leave: Option<bool>,
) -> Result<()> {
    can_manage(ctx, game).await?;

    if !also_leave.unwrap_or_default() {
        query!(
            r#"
            insert
            into players
                (game_id, user_id)
            values
                ($1, (select owner_id from games where id = $1))
            "#,
            game,
        )
        .execute(&ctx.data().pool)
        .await?;
    }

    let record = query!(
        r#"
        update games
        set
            owner_id = $3
        where guild_id = $1 and id = $2
        returning title
        "#,
        ctx.guild_id().unwrap().get() as i64,
        game,
        user.id.get() as i64,
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    query!(
        r#"
        delete
        from players
        where game_id = $1 and user_id = $2
        "#,
        game,
        user.id.get() as i64,
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say(format!(
        "Transferred `{}` to {}",
        record.title,
        user.mention()
    ))
    .await?;

    Ok(())
}
