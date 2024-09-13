use sqlx::query;

use crate::{commands::confirmation_modal, Context, Result};

#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES", ephemeral)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "System to delete"]
    #[autocomplete = "crate::autocomplete::system"]
    system: i32,
) -> Result<()> {
    let maybe_abbreviation = query!(
        r#"
        select
            abbreviation
        from systems
        where id = $1 and guild_id = $2
        "#,
        system,
        ctx.guild_id().unwrap().get() as i64,
    )
    .fetch_optional(&ctx.data().pool)
    .await?
    .map(|r| r.abbreviation);

    match maybe_abbreviation {
        Some(abbreviation) => {
            confirmation_modal()
                .ctx(&ctx)
                .phrase(&abbreviation)
                .failure_message("Confirmation failed. System was not deleted.")
                .success_message("System deleted!")
                .then(|| async {
                    query!(
                        r#"
                        delete
                        from systems
                        where id = $1 and guild_id = $2
                        "#,
                        system,
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
            ctx.say("System not found! Not sure how you got here...")
                .await?;
        }
    }

    Ok(())
}
