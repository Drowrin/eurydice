use poise::CreateReply;
use serenity::all::{EditMember, Member, Mentionable, UserId};
use sqlx::query;

use crate::{
    commands::{contextual_args, game::can_manage},
    Context, Result,
};

fn strip_character_name(member: &Member) -> String {
    member
        .display_name()
        .split("(")
        .next()
        .unwrap()
        .trim_end()
        .to_string()
}

/// Activate this game, reverting all players' nicknames. Usable by game owners and server moderators.
#[poise::command(slash_command)]
pub async fn deactivate(
    ctx: Context<'_>,
    #[description = "The game to deactivate"]
    #[autocomplete = "crate::autocomplete::game_editable"]
    game: Option<i32>,
) -> Result<()> {
    let game = contextual_args()
        .game_id_arg(game)
        .ctx(&ctx)
        .call()
        .await?
        .game_id;

    can_manage(ctx, game).await?;

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

    let mut changes = vec![];

    for player in players {
        let player_id = UserId::from(player.user_id as u64);
        if player_id == owner_id {
            owner_found = true;
            continue;
        }

        let member = ctx.guild_id().unwrap().member(ctx, player_id).await?;
        let nick_name = strip_character_name(&member);

        changes.push(format!("{} --> {}", member.display_name(), nick_name));

        ctx.guild_id()
            .unwrap()
            .edit_member(
                ctx,
                player_id,
                EditMember::new()
                    .nickname(nick_name)
                    .audit_log_reason("Game deactivated by command"),
            )
            .await?;
    }

    ctx.say(format!(
        "`{game_title}` deactivated!\n```\n{}\n```",
        changes.join("\n")
    ))
    .await?;

    if owner_found {
        let owner = ctx.guild_id().unwrap().member(ctx, owner_id).await?;
        let nick_name = strip_character_name(&owner);
        let why = "[Why?](<https://github.com/Drowrin/eurydice/wiki/Why-is-the-bot-telling-me-to-use-a-nick-command>)";

        if owner.display_name() != nick_name {
            if owner_id == ctx.author().id {
                ctx.send(
                    CreateReply::default()
                        .content(format!("You need to run `/nick {nick_name}`\n{why}"))
                        .ephemeral(true),
                )
                .await?;
            } else {
                ctx.say(format!(
                    "{} needs to run `/nick {nick_name}`\n{why}",
                    owner_id.mention(),
                ))
                .await?;
            }
        }
    }

    Ok(())
}
