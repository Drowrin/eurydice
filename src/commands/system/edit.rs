use poise::{CreateReply, Modal};
use sqlx::query;

use crate::{
    commands::system::{system_embed, SystemModal},
    Context, Result,
};

#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES", ephemeral)]
pub async fn edit(
    ctx: Context<'_>,
    #[description = "System to edit"]
    #[autocomplete = "crate::autocomplete::system"]
    system: i32,
) -> Result<()> {
    let maybe_old_system = query!(
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

    let old_system = match maybe_old_system {
        Some(old_system) => old_system,
        _ => {
            ctx.say("System not found! Not sure how you got here...")
                .await?;
            return Ok(());
        }
    };

    let defaults = SystemModal {
        title: old_system.title,
        abbreviation: old_system.abbreviation,
        description: old_system.description,
        image: old_system.image,
    };
    let maybe_system_data = SystemModal::execute_with_defaults(ctx, defaults).await?;

    if let Some(system_data) = maybe_system_data {
        query!(
            r#"
            update systems set
                title = $3,
                abbreviation = $4,
                description = $5,
                image = $6
            where id = $1 and guild_id = $2
            "#,
            system,
            ctx.guild_id().unwrap().get() as i64,
            system_data.title.clone(),
            system_data.abbreviation.clone(),
            system_data.description.clone(),
            system_data.image.clone(),
        )
        .execute(&ctx.data().pool)
        .await?;

        ctx.send(
            CreateReply::default().content("System updated!").embed(
                system_embed()
                    .title(system_data.title)
                    .abbreviation(system_data.abbreviation)
                    .description(system_data.description)
                    .image(system_data.image)
                    .call(),
            ),
        )
        .await?;
    }

    Ok(())
}
