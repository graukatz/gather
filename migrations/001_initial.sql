CREATE TABLE IF NOT EXISTS guild_config (
    guild_id INTEGER PRIMARY KEY,
    forum_id INTEGER,
    lfg_command_channel_id INTEGER
);

CREATE TABLE IF NOT EXISTS lfg_types (
    guild_id INTEGER,
    name TEXT NOT NULL,
    role_id INTEGER NOT NULL,

    PRIMARY KEY(guild_id, name)
);

CREATE TABLE IF NOT EXISTS sessions (
    post_id INTEGER PRIMARY KEY,
    guild_id INTEGER NOT NULL,
    forum_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,

    announcement_channel_id INTEGER NOT NULL,
    announcement_message_id INTEGER NOT NULL,

    complete_message_id INTEGER NOT NULL,

    last_activity_timestamp INTEGER NOT NULL,

    warning_sent BOOLEAN NOT NULL,
    warning_message_id INTEGER,

    permission_warning_sent BOOLEAN NOT NULL,

    is_done BOOLEAN NOT NULL
);