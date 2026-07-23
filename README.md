![Rust](https://img.shields.io/badge/Rust-stable-orange)
![License](https://img.shields.io/github/license/graukatz/gather)

# Gather

Gather quickly organizes multiplayer sessions using Discord Forums.

Set custom Looking For Group (LFG) categories and notification roles while members can quickly create LFG posts with automatic role pings.
Gather reminds users about inactive posts and automatically cleans up completed or abandoned sessions.

## Table of Contents

* [Features & Commands](#features--commands)
* [Build Requirements](#build-requirements)
* [Installation](#installation)
* [Configuration & Setup](#configuration--setup)
* [Permissions](#permissions)
* [License](#license)
* [AI Disclosure](#ai-disclosure)

## Features & Commands

> Note: Gather consists of slash commands

* [Help](#help)
* [LFG Creation](#lfg-creation)
* [Add Type](#add-type)
* [Remove Type](#remove-type)
* [Channel Configuration](#channel-configuration)
* [Check Permissions](#check-permissions)

### Help

Displays available commands, setup instructions and information about inactivity timers.

`/help`

### LFG Creation

Create a new LFG session. An optional message can be provided.

`/lfg-create TYPE PLAYERS (MESSAGE)`

### Add Type

Add a new LFG type via name and the role that should be pinged. Administrator permissions required.

`/gthr-add-type NAME ROLE`

### Remove Type

Remove an existing LFG type via name. Administrator permissions required.

`/gthr-remove-type NAME`

### Channel Configuration

Configures the forum used for LFG posts and the channel where `/lfg-create` must be used. Administrator permissions required.

`/gthr-channel-config FORUM COMMAND`

### Check Permissions

> Note: Even though Gather performs a permission check when you run `/gthr-channel-config FORUM COMMAND`, it is highly recommended that you run the `/gthr-check-permissions` command after every change to permissions to ensure full functionality.

Check if Gather has all permissions it requires. Administrator permissions required.

`/gthr-check-permissions`

## Build Requirements

* [Rust](https://rust-lang.org/tools/install/)
* [A Discord bot application](https://discord.com/developers/applications)
* A Discord server

## Installation

There are two ways to get Gather running

### Public Bot:

* [Invite the bot](https://discord.com/oauth2/authorize?client_id=1525546166979133701)

### Build & host yourself:

Clone the repository

```bash
git clone https://github.com/graukatz/gather.git
cd gather
```

Create a `.env` file (or otherwise provide the DISCORD_TOKEN environment variable)

```text
DISCORD_TOKEN=<token>
```

Run Gather

```bash
cargo run --release
```

or execute the compiled binary from `target/release/`

## Configuration & Setup

1. Invite Gather to your server
2. Configure the LFG forum and command channel

`/gthr-channel-config FORUM COMMAND`

3. Add one or more LFG types

`/gthr-add-type NAME ROLE`

4. Verify the bot has all required permissions

`/gthr-check-permissions`

Gather is now ready to use. Try running `/lfg-create TYPE PLAYERS (MESSAGE)`

## Permissions

Gather requires following Discord permissions:

* Manage Channels
* View Channels
* Send Messages
* Create Public Threads
* Manage Messages
* Manage Threads
* Embed Links
* Read Message History
* Mention @everyone, @here and All Roles

Activity tracking or cleanup might not work as expected or might not work at all if certain permissions are not given.

To correct permission issues and check if Gather is operating normally, use `/gthr-check-permissions`.

## License

This project is licensed under the MIT License

## AI Disclosure

Parts of this project were developed with assistance from AI tools. AI was used as an aid for tasks such as debugging, explaining concepts, and reviewing code. All generated suggestions were reviewed, adapted, and tested before being included.