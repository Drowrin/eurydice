use poise::{CreateReply, FrameworkError};
use sqlx::error::ErrorKind;

use crate::Data;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error("Constraint Error: {0:?}")]
    Constraint(ErrorKind),
    #[error("I couldn't find what you were looking for, not sure how this happened!")]
    NotFound,
    #[error(transparent)]
    Discord(#[from] serenity::all::Error),

    #[error(transparent)]
    Eyre(#[from] eyre::Error),

    #[error("ErrorMessage: {0}")]
    Message(String),
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::Database(e) => {
                let k = e.kind();
                Self::Constraint(k)
            }
            _ => Self::Sqlx(value),
        }
    }
}

fn msg(message: impl AsRef<str>) -> CreateReply {
    CreateReply::default()
        .ephemeral(true)
        .content(message.as_ref())
}

pub async fn handle(error: FrameworkError<'_, Data, Error>) {
    let r: Result<()> = async {
        match error {
            FrameworkError::Command { error, ctx, .. } => match error {
                Error::Message(e) => {
                    ctx.send(msg(e)).await?;
                }
                error => {
                    println!(
                        "Unhandled Error in Command `{}`: {}",
                        error,
                        ctx.command().name
                    );
                    ctx.send(msg("Something went wrong, sorry!")).await?;
                }
            },
            FrameworkError::CommandCheckFailed { ctx, error, .. } => match error {
                Some(err) => {
                    println!(
                        "CommandCheckFailed Error in Command `{}`: {}",
                        err,
                        ctx.command().name
                    );
                    ctx.send(msg("Something went wrong, sorry!")).await?;
                }
                None => {
                    ctx.send(msg("You don't have permission to do that!"))
                        .await?;
                }
            },
            FrameworkError::Setup {
                error,
                framework: _framework,
                data_about_bot: _data_about_bot,
                ctx: _ctx,
                ..
            } => {
                println!("Setup Error: {error}");
            }
            FrameworkError::EventHandler {
                error,
                ctx: _ctx,
                event: _event,
                framework: _framework,
                ..
            } => {
                println!("EventHandler Error: {error}");
            }
            FrameworkError::CommandPanic { payload, ctx, .. } => {
                println!("Command Panic ({}): {:?}", ctx.command().name, payload);
            }
            FrameworkError::ArgumentParse {
                error,
                input: _input,
                ctx: _ctx,
                ..
            } => {
                println!("ArgumentParse Error: {error}");
            }
            FrameworkError::CommandStructureMismatch {
                description,
                ctx: _ctx,
                ..
            } => {
                println!(
                    "CommandStructureMismatch Error (maybe re-publish commands?): {}",
                    description
                );
            }
            FrameworkError::MissingBotPermissions {
                missing_permissions,
                ctx,
                ..
            } => {
                println!(
                    "MissingBotPermissions Error ({}): {}",
                    ctx.command().name,
                    missing_permissions,
                );
            }
            FrameworkError::MissingUserPermissions {
                missing_permissions: _missing_permissions,
                ctx,
                ..
            } => {
                ctx.send(msg("You don't have permission to do that!"))
                    .await?;
            }
            FrameworkError::NotAnOwner { ctx, .. } => {
                ctx.send(msg("You don't have permission to do that!"))
                    .await?;
            }
            FrameworkError::GuildOnly { ctx, .. } => {
                ctx.send(msg("This command can only be run in a server."))
                    .await?;
            }
            FrameworkError::DmOnly { ctx, .. } => {
                ctx.send(msg("This command can only be run in a direct message."))
                    .await?;
            }
            FrameworkError::NsfwOnly { ctx, .. } => {
                ctx.send(msg("This command can only be run in an NSFW channel."))
                    .await?;
            }
            FrameworkError::UnknownInteraction {
                ctx: _ctx,
                framework: _framework,
                interaction,
                ..
            } => {
                println!("UnknownInteraction Error: {:?}", interaction);
            }
            e => poise::builtins::on_error(e).await?,
        }

        Ok(())
    }
    .await;

    if let Err(e) = r {
        println!("Error while handling errors: {}", e);
    }
}
