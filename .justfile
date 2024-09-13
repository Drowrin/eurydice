set windows-shell := ["pwsh", "-NoLogo", "-NoProfileLoadTime", "-Command"]

[private]
@default:
    just --list

[group("database")]
@new-migration name:
    sqlx migrate add -r {{ name }}

[confirm]
[group("database")]
@reset-database:
    sqlx database create
    sqlx migrate revert --target-version 0
    sqlx migrate run

[group("rust")]
@build:
    cargo build

[group("rust")]
@check:
    cargo clippy

[group("rust")]
@test:
    cargo test --message-format short --test db

[group("discord")]
@publish-commands:
    cargo run --bin publish-commands

[group("discord")]
@run:
    cargo run --bin bot
