use poise::CreateReply;
use serenity::all::{EditMember, Member, Mentionable, UserId};
use sqlx::query;

use crate::{
    commands::{contextual_args, game::can_manage},
    Context, Result,
};

fn apply_character_name(member: &Member, character_name: String) -> String {
    let base_name = member.display_name().split("(").next().unwrap().trim_end();
    format!("{base_name} ({character_name})")
}

#[poise::command(slash_command)]
pub async fn activate(
    ctx: Context<'_>,
    #[description = "The game to activate"]
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

    let mut changes = vec![];

    for player in players {
        let player_id = UserId::from(player.user_id as u64);
        if player_id == owner_id {
            owner_character_name = player.character_name;
            continue;
        }

        if let Some(character_name) = player.character_name {
            let member = ctx.guild_id().unwrap().member(ctx, player_id).await?;
            let nick_name = apply_character_name(&member, character_name);

            changes.push(format!("{} --> {}", member.display_name(), nick_name));

            ctx.guild_id()
                .unwrap()
                .edit_member(
                    ctx,
                    player_id,
                    EditMember::new()
                        .nickname(nick_name)
                        .audit_log_reason("Game activated by command"),
                )
                .await?;
        }
    }

    ctx.say(format!(
        "`{game_title}` activated!\n```\n{}\n```",
        changes.join("\n")
    ))
    .await?;

    if let Some(character_name) = owner_character_name {
        let owner = ctx.guild_id().unwrap().member(ctx, owner_id).await?;
        let why = "[Why?](<https://github.com/Drowrin/eurydice/wiki/Why-is-the-bot-telling-me-to-use-a-nick-command>)";
        let nick_name = apply_character_name(&owner, character_name);

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

    Ok(())
}
