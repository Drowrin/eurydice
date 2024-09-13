use std::{future::Future, time::Duration};

use serenity::all::{
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateQuickModal,
};

use crate::{Context, Result};

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
                            .content(success_message.unwrap_or("System deleted!")),
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
