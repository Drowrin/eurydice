use poise::{CreateReply, Modal};
use serenity::all::{ChannelId, RoleId, UserId};
use sqlx::query;

use crate::{
    commands::game::{can_manage, game_embed, GameModal},
    Context, Result,
};

#[poise::command(slash_command, check = "can_manage", ephemeral)]
pub async fn edit(
    ctx: Context<'_>,
    #[description = "The game to edit"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
) -> Result<()> {
    let maybe_old_game = query!(
        r#"
        select
            title, abbreviation, description, image
        from games
        where id = $1 and guild_id = $2
        "#,
        game,
        ctx.guild_id().unwrap().get() as i64,
    )
    .fetch_optional(&ctx.data().pool)
    .await?;

    let old_game = if let Some(old_game) = maybe_old_game {
        old_game
    } else {
        ctx.say("Game not found! Not sure how you got here...")
            .await?;
        return Ok(());
    };

    let defaults = GameModal {
        title: old_game.title,
        abbreviation: old_game.abbreviation,
        description: old_game.description,
        image: old_game.image,
    };
    let maybe_game_data = GameModal::execute_with_defaults(ctx, defaults).await?;

    if let Some(game_data) = maybe_game_data {
        let record = query!(
            r#"
            update games as g set
                title = $3,
                abbreviation = $4,
                description = $5,
                image = $6
            where id = $1 and guild_id = $2
            returning
                created_at,
                role_id,
                owner_id,
                main_channel_id,
                (select abbreviation from systems where id = g.system_id) as "system",
                (select user_id from players where game_id = g.id) as "players"
            "#,
            game,
            ctx.guild_id().unwrap().get() as i64,
            game_data.title.clone(),
            game_data.abbreviation.clone(),
            game_data.description.clone(),
            game_data.image.clone(),
        )
        .fetch_one(&ctx.data().pool)
        .await?;

        ctx.send(
            CreateReply::default().content("Game updated!").embed(
                game_embed()
                    .title(game_data.title)
                    .abbreviation(game_data.abbreviation)
                    .description(game_data.description)
                    .image(game_data.image)
                    .system(record.system)
                    .created_at(record.created_at)
                    .role_id(RoleId::from(record.role_id as u64))
                    .channel_id(record.main_channel_id.map(|c| ChannelId::from(c as u64)))
                    .owner_id(UserId::from(record.owner_id as u64))
                    .players(
                        record
                            .players
                            .into_iter()
                            .map(|p| UserId::from(p as u64))
                            .collect(),
                    )
                    .call(),
            ),
        )
        .await?;
    }

    Ok(())
}
