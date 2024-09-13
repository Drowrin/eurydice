create table if not exists systems (
    id int primary key generated always as identity,
    guild_id bigint not null,

    title text not null,
    abbreviation text not null,
    description text,

    image text,
    link text,

    unique (title, guild_id),
    unique (abbreviation, guild_id)
);

create table if not exists games (
    id int primary key generated always as identity,

    guild_id bigint not null,
    owner_id bigint not null,
    role_id bigint not null,

    main_channel_id bigint,

    title text not null,
    abbreviation text not null,
    description text,

    system_id int references systems(id) on delete set null,

    image text,

    created_at timestamp with time zone not null default (now() at time zone 'utc'),

    unique (title, guild_id),
    unique (abbreviation, guild_id),
    unique (main_channel_id, guild_id)
);

create table if not exists characters (
    id int primary key generated always as identity,
    game_id int not null references games(id) on delete cascade,
    guild_id bigint not null,
    author_id bigint not null,

    name text not null,
    pronouns text,

    image text,

    description text,

    unique (name, game_id)
);

create table if not exists players (
    user_id bigint,
    game_id int references games (id) on delete cascade,

    character_id int references characters(id) on delete set null,

    primary key (user_id, game_id)
);