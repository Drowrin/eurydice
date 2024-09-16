use std::{future::Future, time::Duration};

use serenity::all::{
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateQuickModal,
};
use sqlx::query;

use crate::{Context, Error, Result};

pub mod character;
pub mod game;
pub mod system;

pub fn all() -> Vec<crate::Command> {
    vec![system::system(), game::game(), character::character()]
}

#[bon::builder]
pub async fn confirmation_modal<F, Fut>(
    ctx: &Context<'_>,
    phrase: &str,
    success_message: Option<&str>,
    failure_message: Option<&str>,
    then: F,
) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let phrase = phrase.to_lowercase();

    let modal = CreateQuickModal::new("Are you sure?")
        .timeout(Duration::from_secs(600))
        .short_field(format!("Type \"{phrase}\" to confirm"));

    let response = ctx
        .interaction
        .quick_modal(ctx.serenity_context, modal)
        .await?;

    match response {
        Some(s) if s.inputs[0].to_lowercase() == phrase => {
            then().await?;
            s.interaction
                .create_response(
                    ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(success_message.unwrap_or("Confirmed!")),
                    ),
                )
                .await?;
            Ok(())
        }
        Some(s) => {
            s.interaction
                .create_response(
                    ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content(
                            failure_message
                                .unwrap_or("Confirmation failed. Action was not completed."),
                        ),
                    ),
                )
                .await?;
            Ok(())
        }
        _ => Ok(()),
    }
}

#[derive(Debug)]
pub struct ContextualArgs {
    pub game_id: i32,
    pub character_id: Option<i32>,
}

#[bon::builder]
pub async fn contextual_args(
    ctx: &Context<'_>,
    game_id_arg: Option<Option<i32>>,
    character_id_arg: Option<Option<i32>>,
) -> Result<ContextualArgs> {
    let maybe_game_id: Option<i32> = match game_id_arg {
        Some(Some(game_id)) => Some(game_id),
        None | Some(None) => {
            let maybe_game_data = query!(
                r#"
                select id
                from games
                where
                    guild_id = $1
                    and
                    main_channel_id = $2
                "#,
                ctx.guild_id().unwrap().get() as i64,
                ctx.channel_id().get() as i64,
            )
            .fetch_optional(&ctx.data().pool)
            .await?;
            maybe_game_data.map(|game_data| game_data.id)
        }
    };

    let game_id = match maybe_game_id {
        Some(game_id) => game_id,
        None => match character_id_arg {
            Some(Some(character_id)) => {
                let maybe_player = query!(
                    r#"
                    select game_id
                    from characters
                    where
                        id = $1
                        and
                        guild_id = $2
                    "#,
                    character_id,
                    ctx.guild_id().unwrap().get() as i64,
                )
                .fetch_optional(&ctx.data().pool)
                .await?;

                match maybe_player {
                    Some(player) => player.game_id,
                    _ => return Err(Error::NotFound),
                }
            }
            _ => {
                let arg = if character_id_arg.is_some() {
                    "character"
                } else {
                    "game"
                };

                return Err(Error::Message(
                    [
                        "This is not a game channel.",
                        "I couldn't figure out what you meant by context.",
                        "\nTry using this command from a game channel,",
                        format!("or hit TAB to select a `{arg}`.").as_str(),
                    ]
                    .join(" "),
                ));
            }
        },
    };

    let character_id = match character_id_arg {
        Some(Some(character_id)) => Some(character_id),
        Some(None) => {
            let maybe_player = query!(
                r#"
                select character_id
                from players
                where
                    game_id = $1
                    and
                    user_id = $2
                "#,
                game_id,
                ctx.author().id.get() as i64,
            )
            .fetch_optional(&ctx.data().pool)
            .await?;

            let player = match maybe_player {
                Some(player) => player,
                _ => {
                    return Err(Error::Message(
                        [
                            "You are not a player in this game.",
                            "I couldn't figure out what you meant by context.",
                            "\nTry using this command from a game channel you play in,",
                            "or hit TAB to select a `character`.",
                        ]
                        .join(" "),
                    ));
                }
            };

            match player.character_id {
                Some(character_id) => Some(character_id),
                _ => {
                    return Err(Error::Message(
                        [
                            "You don't have a character assigned for this game.",
                            "I couldn't figure out what you meant by context.",
                            "\nTry using this command from a game channel,",
                            "or hit TAB to select a `character`.",
                        ]
                        .join(" "),
                    ));
                }
            }
        }
        None => None,
    };

    Ok(ContextualArgs {
        game_id,
        character_id,
    })
}
