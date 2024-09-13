<div align="center">

# Eurydice

A Discord bot providing organization tools for TTRPG groups.

</div>

## Features and Roadmap

This project is currently very early in development.

It has the following features planned/implemented:

- [x] Game management
  - [x] Display and edit system details per guild
    - [x] Title, abbreviation, description, and image
    - [x] Only editable by server moderators
  - [x] Display and edit game details
    - [x] Title, abbreviation, description, image, system, and game owner
    - [x] Editable by a game's owner and server moderators
  - [x] Optionally associate a channel with each game
    - [x] Automatically infers command arguments based on channel context
  - [x] Automatically create a role based on game abbreviation
  - [x] Add and remove players from games
  - [x] Transfer ownership of the game to another user, optionally becoming a player
- [x] Character management
  - [x] Display name, description, image, pronouns
  - [x] Decoupled character and player lists
  - [x] Reassign, release, and claim actions.
  - [x] Keep track of original character author, even if reassigned to another player
  - [x] editable by current player, game owner, original author, and server moderators
- [ ] Session management
  - [ ] Keep track of sessions and display using discord events
  - [ ] Allow for postponement or rescheduling
  - [x] Set nicknames of players to character names during a session
  - [ ] Tools for organizing session recap/synopses
  - [ ] Automatic links to live streams
  - [ ] Ready Check
    - [ ] Ping participants of a game ahead of time
    - [ ] Gather responses from participants
    - [ ] Show results at a glance
- [ ] Safety tool management
  - [ ] Keep track of safety tool information per-game
  - [ ] Easy, anonymous additions
