use sqlx::{Pool, Postgres};

pub mod autocomplete;
pub mod commands;

pub mod error;

pub use error::{Error, Result};

pub type Context<'a> = poise::ApplicationContext<'a, Data, Error>;
pub type Command = poise::Command<Data, Error>;
pub type DB = Pool<Postgres>;

#[derive(Debug)]
pub struct Data {
    pub pool: DB,
}
