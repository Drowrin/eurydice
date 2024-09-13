use serenity::all::{EditMember, Member, Mentionable, UserId};
use sqlx::query;

use crate::{commands::game::can_manage, Context, Result};

fn strip_character_name(member: Member) -> String {
    member
        .display_name()
        .split("(")
        .next()
        .unwrap()
        .trim_end()
        .to_string()
}

#[poise::command(slash_command, check = "can_manage")]
pub async fn deactivate(
    ctx: Context<'_>,
    #[description = "The game to deactivate"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
) -> Result<()> {
    let players = query!(
        r#"
        select
            user_id
        from players
        where game_id = $1
        "#,
        game
    )
    .fetch_all(&ctx.data().pool)
    .await?;

    let game_title = query!(
        r#"
        select title
        from games
        where id = $1
        "#,
        game
    )
    .fetch_one(&ctx.data().pool)
    .await?
    .title;

    let owner_id = ctx.guild().unwrap().owner_id;
    let mut owner_found: bool = false;

    for player in players {
        let player_id = UserId::from(player.user_id as u64);
        if player_id == owner_id {
            owner_found = true;
            continue;
        }

        let member = ctx.guild_id().unwrap().member(ctx, player_id).await?;

        ctx.guild_id()
            .unwrap()
            .edit_member(
                ctx,
                player_id,
                EditMember::new()
                    .nickname(strip_character_name(member))
                    .audit_log_reason("Game deactivated by command"),
            )
            .await?;
    }

    ctx.say(format!("`{game_title}` deactivated!")).await?;

    if owner_found {
        let owner = ctx.guild_id().unwrap().member(ctx, owner_id).await?;
        let stripped_name = strip_character_name(owner.clone());
        if owner.display_name() != stripped_name {
            ctx.say(format!(
                "{} needs to run `/nick {}`",
                owner_id.mention(),
                strip_character_name(owner)
            ))
            .await?;
        }
    }

    Ok(())
}
