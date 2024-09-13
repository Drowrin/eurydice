use serenity::all::{Member, Mentionable};
use sqlx::query;

use crate::{commands::game::can_manage, Context, Result};

#[poise::command(slash_command, check = "can_manage")]
pub async fn add(
    ctx: Context<'_>,
    #[description = "The game to add a player to"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
    #[description = "The user to add to the game"] user: Member,
) -> Result<()> {
    let owner_id = query!(
        r#"
        select owner_id
        from games
        where guild_id = $1 and id = $2
        "#,
        ctx.guild_id().unwrap().get() as i64,
        game,
    )
    .fetch_one(&ctx.data().pool)
    .await?
    .owner_id;

    if owner_id == user.user.id.get() as i64 {
        ctx.say("Can't add game owner as a player.").await?;
    }

    let record = query!(
        r#"
        insert
        into players (user_id, game_id)
        values ($1, $2)
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
                "Player {} added to `{}`!",
                user.mention(),
                record.title.unwrap()
            ))
            .await?;
        }
        Err(sqlx::Error::Database(_)) => {
            ctx.say(format!("{} is already in that game.", user.mention()))
                .await?;
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
