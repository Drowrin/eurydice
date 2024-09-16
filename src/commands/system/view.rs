use poise::CreateReply;
use sqlx::query;

use crate::{commands::system::system_embed, Context, Result};

/// View a system's details. Usable by everyone.
#[poise::command(slash_command)]
pub async fn view(
    ctx: Context<'_>,
    #[description = "System to view"]
    #[autocomplete = "crate::autocomplete::system"]
    system: i32,
) -> Result<()> {
    let maybe_system = query!(
        r#"
        select
            title, abbreviation, description, image
        from systems
        where id = $1 and guild_id = $2
        "#,
        system,
        ctx.guild_id().unwrap().get() as i64,
    )
    .fetch_optional(&ctx.data().pool)
    .await?;

    match maybe_system {
        Some(system) => {
            ctx.send(
                CreateReply::default().embed(
                    system_embed()
                        .title(system.title)
                        .abbreviation(system.abbreviation)
                        .description(system.description)
                        .image(system.image)
                        .call(),
                ),
            )
            .await?;
        }
        None => {
            ctx.say("System not found! Not sure how you got here...")
                .await?;
        }
    }

    Ok(())
}
