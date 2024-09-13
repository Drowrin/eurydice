use serenity::all::{EditMember, Member, Mentionable, UserId};
use sqlx::query;

use crate::{commands::game::can_manage, Context, Result};

fn apply_character_name(member: Member, character_name: String) -> String {
    let base_name = member.display_name().split("(").next().unwrap().trim_end();
    format!("{base_name} ({character_name})")
}

#[poise::command(slash_command, check = "can_manage")]
pub async fn activate(
    ctx: Context<'_>,
    #[description = "The game to activate"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: i32,
) -> Result<()> {
    let players = query!(
        r#"
        select
            user_id,
            (select name from characters where id = character_id) as "character_name"
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
    let mut owner_character_name: Option<String> = None;

    for player in players {
        let player_id = UserId::from(player.user_id as u64);
        if player_id == owner_id {
            owner_character_name = player.character_name;
            continue;
        }

        if let Some(character_name) = player.character_name {
            let member = ctx.guild_id().unwrap().member(ctx, player_id).await?;

            ctx.guild_id()
                .unwrap()
                .edit_member(
                    ctx,
                    player_id,
                    EditMember::new()
                        .nickname(apply_character_name(member, character_name))
                        .audit_log_reason("Game activated by command"),
                )
                .await?;
        }
    }

    ctx.say(format!("`{game_title}` activated!")).await?;

    if let Some(character_name) = owner_character_name {
        let owner = ctx.guild_id().unwrap().member(ctx, owner_id).await?;
        ctx.say(format!(
            "{} needs to run `/nick {}`",
            owner_id.mention(),
            apply_character_name(owner, character_name)
        ))
        .await?;
    }

    Ok(())
}
