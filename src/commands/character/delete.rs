use sqlx::query;

use crate::{
    commands::{character::can_manage, confirmation_modal},
    Context, Result,
};

#[poise::command(slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "The character to delete"]
    #[autocomplete = "crate::autocomplete::character_editable"]
    character: i32,
) -> Result<()> {
    can_manage(ctx, character).await?;

    let maybe_name = query!(
        r#"
        select
            name
        from characters
        where id = $1 and guild_id = $2
        "#,
        character,
        ctx.guild_id().unwrap().get() as i64,
    )
    .fetch_optional(&ctx.data().pool)
    .await?
    .map(|r| r.name);

    match maybe_name {
        Some(name) => {
            confirmation_modal()
                .ctx(&ctx)
                .phrase(&name)
                .failure_message("Confirmation failed. Character was not deleted.")
                .success_message("Character deleted!")
                .then(|| async {
                    query!(
                        r#"
                        delete
                        from characters
                        where id = $1 and guild_id = $2
                        "#,
                        character,
                        ctx.guild_id().unwrap().get() as i64,
                    )
                    .execute(&ctx.data().pool)
                    .await?;
                    Ok(())
                })
                .call()
                .await?;
        }
        None => {
            ctx.say("Character not found! Not sure how you got here...")
                .await?;
        }
    }

    Ok(())
}
