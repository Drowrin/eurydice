use sqlx::query;

use crate::{
    commands::{contextual_args, game::can_manage},
    Context, Result,
};

/// Remove this game's association with any channel.
#[poise::command(slash_command, ephemeral)]
pub async fn unset(
    ctx: Context<'_>,
    #[description = "The game to unset the channel of"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: Option<i32>,
) -> Result<()> {
    let game = contextual_args()
        .game_id_arg(game)
        .ctx(&ctx)
        .call()
        .await?
        .game_id;

    can_manage(ctx, game).await?;

    let record = query!(
        r#"
        update games
        set main_channel_id = null
        where guild_id = $1 and id = $2
        returning title
        "#,
        ctx.guild_id().unwrap().get() as i64,
        game,
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    ctx.say(format!("Unset channel of `{}`.", record.title))
        .await?;

    Ok(())
}
