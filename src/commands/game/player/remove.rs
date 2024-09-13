use serenity::all::{Member, Mentionable};
use sqlx::query;

use crate::{commands::game::can_manage, Context, Result};

#[poise::command(slash_command, check = "can_manage")]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "The game to add a player to"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
    #[description = "The user to remove from the game"] user: Member,
) -> Result<()> {
    let record = query!(
        r#"
        delete
        from players
        where user_id = $1 and game_id = $2
        returning (select title from games where id = $2)
        "#,
        user.user.id.get() as i64,
        game,
    )
    .fetch_one(&ctx.data().pool)
    .await;

    match record {
        Ok(record) => {
            ctx.say(format!(
                "Player {} removed from {}!",
                user.mention(),
                record.title.unwrap()
            ))
            .await?;
        }
        Err(sqlx::Error::RowNotFound) => {
            ctx.say(format!(
                "{} can't be removed from a game they aren't in.",
                user.mention()
            ))
            .await?;
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
