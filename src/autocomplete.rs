use serenity::all::AutocompleteChoice;
use sqlx::query;

use crate::Context;

fn search_terms(partial: &str) -> String {
    format!(
        "\"{}\":*",
        partial.split_whitespace().collect::<Vec<&str>>().join("|")
    )
}

async fn is_mod(ctx: &Context<'_>) -> bool {
    ctx.author_member()
        .await
        .unwrap()
        .permissions
        .unwrap()
        .manage_messages()
}

pub async fn system(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    query!(
        r#"
        select
            id, title
        from systems
        where
            guild_id = $1
            and (
                $3 = ''
                or
                to_tsvector(title) @@ to_tsquery($2)
                or
                to_tsvector(abbreviation) @@ to_tsquery($2)
            )
        limit 25
        "#,
        ctx.guild_id().unwrap().get() as i64,
        search_terms(partial),
        partial,
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
    .into_iter()
    .map(|record| AutocompleteChoice::new(record.title, record.id))
    .collect()
}

pub async fn game(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    query!(
        r#"
        select
            id, title
        from games
        where
            guild_id = $1
            and (
                $3 = ''
                or
                to_tsvector(title) @@ to_tsquery($2)
                or
                to_tsvector(abbreviation) @@ to_tsquery($2)
            )
        limit 25
        "#,
        ctx.guild_id().unwrap().get() as i64,
        search_terms(partial),
        partial,
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
    .into_iter()
    .map(|record| AutocompleteChoice::new(record.title, record.id))
    .collect()
}

pub async fn game_editable(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    if is_mod(&ctx).await {
        return game(ctx, partial).await;
    }

    query!(
        r#"
        select
            id, title
        from games
        where
            guild_id = $1
            and
            owner_id = $4
            and (
                $3 = ''
                or
                to_tsvector(title) @@ to_tsquery($2)
                or
                to_tsvector(abbreviation) @@ to_tsquery($2)
            )
        limit 25
        "#,
        ctx.guild_id().unwrap().get() as i64,
        search_terms(partial),
        partial,
        ctx.author().id.get() as i64,
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
    .into_iter()
    .map(|record| AutocompleteChoice::new(record.title, record.id))
    .collect()
}

pub async fn game_joined(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    query!(
        r#"
        select
            id, title
        from games as g
        where
            guild_id = $1
            and
            (
                owner_id = $4
                or
                exists (
                    select 1
                    from players
                    where
                        user_id = $4
                        and
                        game_id = g.id
                )
            )
            and
            (
                $3 = ''
                or
                to_tsvector(title) @@ to_tsquery($2)
                or
                to_tsvector(abbreviation) @@ to_tsquery($2)
            )
        limit 25
        "#,
        ctx.guild_id().unwrap().get() as i64,
        search_terms(partial),
        partial,
        ctx.author().id.get() as i64,
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
    .into_iter()
    .map(|record| AutocompleteChoice::new(record.title, record.id))
    .collect()
}

pub async fn character(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    query!(
        r#"
        select
            id, name
        from characters
        where
            guild_id = $1
            and (
                $3 = ''
                or
                to_tsvector(name) @@ to_tsquery($2)
            )
        limit 25
        "#,
        ctx.guild_id().unwrap().get() as i64,
        search_terms(partial),
        partial,
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
    .into_iter()
    .map(|record| AutocompleteChoice::new(record.name, record.id))
    .collect()
}

pub async fn character_editable(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    if is_mod(&ctx).await {
        return character(ctx, partial).await;
    }

    query!(
        r#"
        select
            id, name
        from characters as c
        where
            guild_id = $1
            and (
                author_id = $4
                or
                exists (
                    select 1
                    from players as p
                    where
                        p.character_id = c.id
                        and
                        p.user_id = $4
                )
                or
                exists (
                    select 1
                    from games as g
                    where
                        g.id = c.game_id
                        and
                        g.owner_id = $4
                )
            )
            and (
                $3 = ''
                or
                to_tsvector(name) @@ to_tsquery($2)
            )
        limit 25
        "#,
        ctx.guild_id().unwrap().get() as i64,
        search_terms(partial),
        partial,
        ctx.author().id.get() as i64,
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
    .into_iter()
    .map(|record| AutocompleteChoice::new(record.name, record.id))
    .collect()
}

pub async fn character_assigned(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    query!(
        r#"
        select
            id, name
        from characters as c
        where
            guild_id = $1
            and
            exists (
                select 1
                from players as p
                where
                    p.character_id = c.id
                    and
                    p.user_id = $4
            )
            and (
                $3 = ''
                or
                to_tsvector(name) @@ to_tsquery($2)
            )
        limit 25
        "#,
        ctx.guild_id().unwrap().get() as i64,
        search_terms(partial),
        partial,
        ctx.author().id.get() as i64,
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
    .into_iter()
    .map(|record| AutocompleteChoice::new(record.name, record.id))
    .collect()
}

pub async fn character_claimable(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    query!(
        r#"
        select
            id, name
        from characters as c
        where
            guild_id = $1
            and
            not exists (
                select 1
                from players as p
                where
                    p.character_id = c.id
            )
            and
            exists (
                select 1
                from games as g
                where
                    g.id = c.game_id
                    and
                    exists (
                        select 1
                        from players as p
                        where
                            p.game_id = g.id
                            and
                            p.user_id = $4
                    )
            )
            and (
                $3 = ''
                or
                to_tsvector(name) @@ to_tsquery($2)
            )
        limit 25
        "#,
        ctx.guild_id().unwrap().get() as i64,
        search_terms(partial),
        partial,
        ctx.author().id.get() as i64,
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
    .into_iter()
    .map(|record| AutocompleteChoice::new(record.name, record.id))
    .collect()
}
