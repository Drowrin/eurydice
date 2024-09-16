use serenity::all::{Member, Mentionable};
use sqlx::query;

use crate::{
    commands::{confirmation_modal, game::can_manage},
    Context, Result,
};

/// Transfer ownership of this game to another user. Usable by game owners and server moderators.
#[poise::command(slash_command)]
pub async fn transfer(
    ctx: Context<'_>,
    #[description = "The game to transfer"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
    #[description = "The user to transfer ownership of the game to"] user: Member,
    #[description = "Cause owner to also leave the game as a player"] also_leave: Option<bool>,
) -> Result<()> {
    can_manage(ctx, game).await?;

    let game_data = query!(
        r#"
        select role_id, abbreviation
        from games
        where guild_id = $1 and id = $2
        "#,
        ctx.guild_id().unwrap().get() as i64,
        game,
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    confirmation_modal()
        .ctx(&ctx)
        .phrase(&game_data.abbreviation)
        .failure_message("Confirmation failed. Game was not deleted.")
        .success_message("Game deleted!")
        .then(|| async {
            if also_leave.unwrap_or_default() {
                ctx.author_member()
                    .await
                    .unwrap()
                    .remove_role(ctx, game_data.role_id as u64)
                    .await?;
            } else {
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
                user.user.id.get() as i64,
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
                user.user.id.get() as i64,
            )
            .execute(&ctx.data().pool)
            .await?;

            user.add_role(ctx, game_data.role_id as u64).await?;

            ctx.say(format!(
                "Transferred `{}` to {}",
                record.title,
                user.mention()
            ))
            .await?;

            Ok(())
        })
        .call()
        .await?;

    Ok(())
}
