use sqlx::query;

use crate::{
    commands::{confirmation_modal, contextual_args},
    Context, Result,
};

/// Delete a game. Usable by game owners and server moderators.
#[poise::command(slash_command, ephemeral)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "The game to delete"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: Option<i32>,
) -> Result<()> {
    let game = contextual_args()
        .game_id_arg(game)
        .ctx(&ctx)
        .call()
        .await?
        .game_id;

    let maybe_game_data = query!(
        r#"
        select
            abbreviation, role_id
        from games
        where id = $1 and guild_id = $2
        "#,
        game,
        ctx.guild_id().unwrap().get() as i64,
    )
    .fetch_optional(&ctx.data().pool)
    .await?;

    match maybe_game_data {
        Some(game_data) => {
            confirmation_modal()
                .ctx(&ctx)
                .phrase(&game_data.abbreviation)
                .failure_message("Confirmation failed. Game was not deleted.")
                .success_message("Game deleted!")
                .then(|| async {
                    query!(
                        r#"
                        delete
                        from games
                        where id = $1 and guild_id = $2
                        "#,
                        game,
                        ctx.guild_id().unwrap().get() as i64,
                    )
                    .execute(&ctx.data().pool)
                    .await?;

                    ctx.guild_id()
                        .unwrap()
                        .delete_role(ctx, game_data.role_id as u64)
                        .await?;

                    Ok(())
                })
                .call()
                .await?;
        }
        None => {
            ctx.say("Game not found! Not sure how you got here...")
                .await?;
        }
    }

    Ok(())
}
