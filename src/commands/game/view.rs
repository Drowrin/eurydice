use poise::CreateReply;
use serenity::all::{ChannelId, RoleId, UserId};
use sqlx::query;

use crate::{
    commands::{contextual_args, game::game_embed},
    Context, Result,
};

/// View a game's details. Usable by everyone.
#[poise::command(slash_command)]
pub async fn view(
    ctx: Context<'_>,
    #[description = "The game to view"]
    #[autocomplete = "crate::autocomplete::game"]
    game: Option<i32>,
) -> Result<()> {
    let game = contextual_args()
        .game_id_arg(game)
        .ctx(&ctx)
        .call()
        .await?
        .game_id;

    let maybe_game = query!(
        r#"
        select
            title, abbreviation, description, image,
            created_at, role_id, owner_id, main_channel_id,
            (select abbreviation from systems where id = g.system_id) as "system"
        from games as g
        where id = $1 and guild_id = $2
        "#,
        game,
        ctx.guild_id().unwrap().get() as i64,
    )
    .fetch_optional(&ctx.data().pool)
    .await?;

    let players = query!(
        r#"
        select user_id
        from players
        where game_id = $1
        "#,
        game
    )
    .fetch_all(&ctx.data().pool)
    .await?;

    match maybe_game {
        Some(game) => {
            ctx.send(
                CreateReply::default().embed(
                    game_embed()
                        .title(game.title)
                        .abbreviation(game.abbreviation)
                        .description(game.description)
                        .image(game.image)
                        .system(game.system)
                        .created_at(game.created_at)
                        .role_id(RoleId::from(game.role_id as u64))
                        .channel_id(game.main_channel_id.map(|c| ChannelId::from(c as u64)))
                        .owner_id(UserId::from(game.owner_id as u64))
                        .players(
                            players
                                .into_iter()
                                .map(|p| UserId::from(p.user_id as u64))
                                .collect(),
                        )
                        .call(),
                ),
            )
            .await?;
        }
        None => {
            ctx.say("Game not found! Not sure how you got here...")
                .await?;
        }
    }

    Ok(())
}
