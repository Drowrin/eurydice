use poise::{CreateReply, Modal};
use sqlx::query;

use crate::{
    commands::system::{system_embed, SystemModal},
    Context, Result,
};

/// Create a system that can be used by games in this server. Usable by server moderators.
#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES", ephemeral)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Title of the system"]
    #[min_length = 3]
    #[max_length = 100]
    title: String,
    #[description = "Abbreviation of the title"]
    #[min_length = 3]
    #[max_length = 32]
    abbreviation: String,
) -> Result<()> {
    let already_exists = query!(
        r#"
        select exists(
            select 1
            from systems
            where
                guild_id = $2
                and
                (
                    title = $1
                    or
                    abbreviation = $1
                )
        )
        "#,
        title,
        ctx.guild_id().unwrap().get() as i64,
    )
    .fetch_one(&ctx.data().pool)
    .await?
    .exists
    .unwrap();

    if already_exists {
        ctx.reply(format!("`{title}` already exists!")).await?;
        return Ok(());
    }

    let defaults = SystemModal {
        abbreviation,
        title,
        ..Default::default()
    };
    let maybe_system_data = SystemModal::execute_with_defaults(ctx, defaults).await?;

    if let Some(system_data) = maybe_system_data {
        query!(
            r#"
            insert
            into systems
                (guild_id, title, abbreviation, description, image)
            values
                ($1, $2, $3, $4, $5)
            "#,
            ctx.guild_id().unwrap().get() as i64,
            system_data.title.clone(),
            system_data.abbreviation.clone(),
            system_data.description.clone(),
            system_data.image.clone(),
        )
        .execute(&ctx.data().pool)
        .await?;

        ctx.send(
            CreateReply::default().content("System created!").embed(
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
