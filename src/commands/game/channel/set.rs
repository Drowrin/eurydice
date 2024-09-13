use serenity::all::{Channel, Mentionable};
use sqlx::query;

use crate::{commands::game::can_manage, Context, Result};

#[poise::command(slash_command, ephemeral)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "The game to set the channel of"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
    #[description = "Channel that will be associated with the game"]
    #[channel_types("Text")]
    channel: Channel,
) -> Result<()> {
    can_manage(ctx, game).await?;

    let record = query!(
        r#"
        update games
        set main_channel_id = $3
        where guild_id = $1 and id = $2
        returning title
        "#,
        ctx.guild_id().unwrap().get() as i64,
        game,
        channel.id().get() as i64,
    )
    .fetch_one(&ctx.data().pool)
    .await;

    match record {
        Ok(record) => {
            ctx.say(format!(
                "Set channel of `{}` to {}.",
                record.title,
                channel.mention(),
            ))
            .await?;

            Ok(())
        }
        Err(sqlx::Error::Database(_)) => {
            ctx.say(format!(
                "Another game has already taken {}.",
                channel.mention(),
            ))
            .await?;

            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
