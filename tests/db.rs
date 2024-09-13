use std::env;

use dotenv::dotenv;
use sqlx::{
    migrate, migrate::MigrateDatabase, postgres::PgPoolOptions, query, Pool, Postgres, Transaction,
};
use tokio::sync::OnceCell;

static INIT: OnceCell<Pool<Postgres>> = OnceCell::const_new();

async fn setup<'a>() -> eurydice::Result<Transaction<'a, Postgres>> {
    Ok(INIT
        .get_or_init(|| async {
            dotenv().unwrap();

            let db_url = format!("{}_tests", env::var("DATABASE_URL").unwrap());

            if Postgres::database_exists(&db_url).await.unwrap() {
                Postgres::drop_database(&db_url).await.unwrap();
            }
            Postgres::create_database(&db_url).await.unwrap();

            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&db_url)
                .await
                .unwrap();

            migrate!("./migrations").run(&pool).await.unwrap();

            pool
        })
        .await
        .begin()
        .await?)
}

#[tokio::test]
async fn unique_system_title() -> eurydice::Result<()> {
    let mut txn = setup().await?;

    query!(
        r#"
        insert into systems
            (guild_id, title, abbreviation)
        values
            ($1, $2, $3)
        "#,
        0,
        "Blades in the Dark",
        "BitD",
    )
    .execute(&mut *txn)
    .await?;

    let result = query!(
        r#"
        insert into systems
            (guild_id, title, abbreviation)
        values
            ($1, $2, $3)
        "#,
        0,
        "Blades in the Dark",
        "different abbreviation",
    )
    .execute(&mut *txn)
    .await;

    assert!(matches!(
        result,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation()
    ));

    Ok(())
}

#[tokio::test]
async fn unique_system_abbreviation() -> eurydice::Result<()> {
    let mut txn = setup().await?;

    query!(
        r#"
        insert into systems
            (guild_id, title, abbreviation)
        values
            ($1, $2, $3)
        "#,
        0,
        "Blades in the Dark",
        "BitD",
    )
    .execute(&mut *txn)
    .await?;

    let result = query!(
        r#"
        insert into systems
            (guild_id, title, abbreviation)
        values
            ($1, $2, $3)
        "#,
        0,
        "different title",
        "BitD",
    )
    .execute(&mut *txn)
    .await;

    assert!(matches!(
        result,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation()
    ));

    Ok(())
}

#[tokio::test]
async fn unique_game_name() -> eurydice::Result<()> {
    let mut txn = setup().await?;

    query!(
        r#"
        insert into games
            (guild_id, owner_id, role_id, title, abbreviation)
        values
            ($1, $2, $3, $4, $5)
        "#,
        0,
        0,
        0,
        "Blades in the Dark",
        "BitD",
    )
    .execute(&mut *txn)
    .await?;

    let result = query!(
        r#"
        insert into games
            (guild_id, owner_id, role_id, title, abbreviation)
        values
            ($1, $2, $3, $4, $5)
        "#,
        0,
        0,
        0,
        "Blades in the Dark",
        "different abbreviation",
    )
    .execute(&mut *txn)
    .await;

    assert!(matches!(
        result,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation()
    ));

    Ok(())
}

#[tokio::test]
async fn unique_game_abbreviation() -> eurydice::Result<()> {
    let mut txn = setup().await?;

    query!(
        r#"
        insert into games
            (guild_id, owner_id, role_id, title, abbreviation)
        values
            ($1, $2, $3, $4, $5)
        "#,
        0,
        0,
        0,
        "Blades in the Dark",
        "BitD",
    )
    .execute(&mut *txn)
    .await?;

    let result = query!(
        r#"
        insert into games
            (guild_id, owner_id, role_id, title, abbreviation)
        values
            ($1, $2, $3, $4, $5)
        "#,
        0,
        0,
        0,
        "different name",
        "BitD",
    )
    .execute(&mut *txn)
    .await;

    assert!(matches!(
        result,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation()
    ));

    Ok(())
}

#[tokio::test]
async fn unique_game_channel() -> eurydice::Result<()> {
    let mut txn = setup().await?;

    query!(
        r#"
        insert into games
            (guild_id, owner_id, role_id, title, abbreviation, main_channel_id)
        values
            ($1, $2, $3, $4, $5, $6)
        "#,
        0,
        0,
        0,
        "Blades in the Dark",
        "BitD",
        0,
    )
    .execute(&mut *txn)
    .await?;

    let result = query!(
        r#"
        insert into games
            (guild_id, owner_id, role_id, title, abbreviation, main_channel_id)
        values
            ($1, $2, $3, $4, $5, $6)
        "#,
        0,
        0,
        0,
        "different name",
        "different abbreviation",
        0
    )
    .execute(&mut *txn)
    .await;

    assert!(matches!(
        result,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation()
    ));

    Ok(())
}

#[tokio::test]
async fn unique_character() -> eurydice::Result<()> {
    let mut txn = setup().await?;

    let game_id = query!(
        r#"
        insert into games
            (guild_id, owner_id, role_id, title, abbreviation)
        values
            ($1, $2, $3, $4, $5)
        returning id
        "#,
        0,
        0,
        0,
        "Blades in the Dark",
        "BitD",
    )
    .fetch_one(&mut *txn)
    .await?
    .id;

    query!(
        r#"
        insert into characters
            (game_id, guild_id, author_id, name)
        values
            ($1, $2, $3, $4)
        "#,
        game_id,
        0,
        0,
        "BitD",
    )
    .execute(&mut *txn)
    .await?;

    let result = query!(
        r#"
        insert into characters
            (game_id, guild_id, author_id, name)
        values
            ($1, $2, $3, $4)
        "#,
        game_id,
        0,
        0,
        "BitD",
    )
    .execute(&mut *txn)
    .await;

    assert!(matches!(
        result,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation()
    ));

    Ok(())
}
