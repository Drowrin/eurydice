use sqlx::query;

use crate::{
    commands::{contextual_args, game::can_manage},
    Context, Result,
};

#[poise::command(slash_command, ephemeral)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "The system to set"]
    #[autocomplete = "crate::autocomplete::system"]
    system: i32,
    #[description = "The game to set the system of"]
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
