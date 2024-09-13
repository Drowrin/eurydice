use poise::{CreateReply, Modal};
use serenity::all::{Channel, EditRole, Mentionable};
use sqlx::query;

use crate::{
    commands::game::{game_embed, GameModal},
    Context, Result,
};

#[poise::command(slash_command, ephemeral)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Title of the game to create"]
    #[min_length = 3]
    #[max_length = 100]
    title: String,
    #[description = "Abbreviation of the game's title, used for the role name"]
    #[min_length = 3]
    #[max_length = 32]
    abbreviation: String,
    #[description = "The system the game will be using"]
    #[autocomplete = "crate::autocomplete::system"]
    system: Option<i32>,
    #[description = "Channel that will be associated with the game"]
    #[channel_types("Text")]
    channel: Option<Channel>,
) -> Result<()> {
    let maybe_already_exists = query!(
        r#"
        select
            title, abbreviation, main_channel_id
        from games
        where
            guild_id = $2
            and
            (
                title = $1
                or
                abbreviation = $1
                or
                main_channel_id = $3
            )
        "#,
        title,
        ctx.guild_id().unwrap().get() as i64,
        channel.clone().map(|c| c.id().get() as i64)
    )
    .fetch_optional(&ctx.data().pool)
    .await?;

    if let Some(already_exists) = maybe_already_exists {
        if title == already_exists.title {
            ctx.reply(format!("`{title}` already exists!")).await?;
        } else if abbreviation == already_exists.abbreviation {
            ctx.reply(format!("`{abbreviation}` is already taken!"))
                .await?;
        } else if channel.clone().map(|c| c.id().get() as i64) == already_exists.main_channel_id {
            let var_name = format!("{} is already taken!", channel.unwrap().mention());
            ctx.reply(var_name).await?;
        }
        return Ok(());
    }

    let defaults = GameModal {
        abbreviation,
        title,
        ..Default::default()
    };
    let maybe_game_data = GameModal::execute_with_defaults(ctx, defaults).await?;

    if let Some(game_data) = maybe_game_data {
        let role = ctx
            .guild_id()
            .unwrap()
            .create_role(
                ctx,
                EditRole::new()
                    .name(game_data.abbreviation.clone())
                    .audit_log_reason("Game role created")
                    .mentionable(true),
            )
            .await?;

        let returned_game_data = query!(
            r#"
            insert
            into games
                (
                    title, abbreviation, description, image,
                    guild_id, owner_id, role_id, system_id,
                    main_channel_id
                )
            values
                ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            returning
                created_at,
                (select abbreviation from systems where id = $8) as "system"
            "#,
            game_data.title.clone(),
            game_data.abbreviation.clone(),
            game_data.description.clone(),
            game_data.image.clone(),
            ctx.guild_id().unwrap().get() as i64,
            ctx.author().id.get() as i64,
            role.id.get() as i64,
            system,
            channel.clone().map(|c| c.id().get() as i64),
        )
        .fetch_one(&ctx.data().pool)
        .await?;

        ctx.send(
            CreateReply::default().content("Game created!").embed(
                game_embed()
                    .title(game_data.title)
                    .abbreviation(game_data.abbreviation)
                    .description(game_data.description)
                    .image(game_data.image)
                    .system(returned_game_data.system)
                    .created_at(returned_game_data.created_at)
                    .role_id(role.id)
                    .channel_id(channel.map(|c| c.id()))
                    .owner_id(ctx.author().id)
                    .players(vec![])
                    .call(),
            ),
        )
        .await?;
    }

    Ok(())
}
