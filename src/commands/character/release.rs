use sqlx::query;

use crate::{
    commands::{character::can_manage, contextual_args},
    Context, Result,
};

/// Unassign a character. Usable by a character's player/author, server moderators, and game owners.
#[poise::command(slash_command)]
pub async fn release(
    ctx: Context<'_>,
    #[description = "The character to release"]
    #[autocomplete = "crate::autocomplete::character_assigned"]
    character: Option<i32>,
) -> Result<()> {
    let character = contextual_args()
        .character_id_arg(character)
        .ctx(&ctx)
        .call()
        .await?
        .character_id
        .unwrap();

    can_manage(ctx, character).await?;

    let record = query!(
        r#"
        update players
        set character_id = null
        where character_id = $1
        returning (select name from characters where id = $1)
        "#,
        character
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    ctx.say(format!("`{}` released.", record.name.unwrap()))
        .await?;

    Ok(())
}
