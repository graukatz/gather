use chrono::Utc;
use poise::serenity_prelude::{ChannelId, GuildId, MessageId, UserId};
use sqlx::Row;
use sqlx::sqlite::SqliteRow;
use crate::data::Error;

#[derive(Clone)]
pub struct Session {
    pub post_id: ChannelId,
    pub guild_id: GuildId,
    pub forum_id: ChannelId,
    pub author_id: UserId,
    pub announcement_channel_id: ChannelId,
    pub announcement_message_id: MessageId,
    pub complete_message_id: MessageId,
    pub last_activity_timestamp: i64,
    pub warning_sent: bool,
    pub permission_warning_sent: bool,
    pub warning_message_id: Option<MessageId>,
    pub is_done: bool
}
impl Session {
    pub fn new(
        post_id: ChannelId,
        guild_id: GuildId,
        forum_id: ChannelId,
        author_id: UserId,
        announcement_channel_id: ChannelId,
        announcement_message_id: MessageId,
        complete_message_id: MessageId
    ) -> Self {
        Self {
            post_id,
            guild_id,
            forum_id,
            author_id,
            announcement_channel_id,
            announcement_message_id,
            complete_message_id,
            last_activity_timestamp: Utc::now().timestamp(),
            warning_sent: false,
            warning_message_id: None,
            permission_warning_sent: false,
            is_done: false
        }
    }

    pub fn from_row(row: &SqliteRow) -> Result<Self, Error> {
        Ok(Self {
            post_id: ChannelId::new(row.get("post_id")),
            guild_id: GuildId::new(row.get("guild_id")),
            forum_id: ChannelId::new(row.get("forum_id")),
            author_id: UserId::new(row.get("author_id")),
            announcement_channel_id: ChannelId::new(
                row.get("announcement_channel_id")
            ),
            announcement_message_id: MessageId::new(
                row.get("announcement_message_id")
            ),
            complete_message_id: MessageId::new(
                row.get("complete_message_id")
            ),
            last_activity_timestamp: row.get("last_activity_timestamp"),
            warning_sent: row.get("warning_sent"),
            warning_message_id:
            row.get::<Option<i64>, _>("warning_message_id")
                .map(|id| MessageId::new(id as u64)),
            permission_warning_sent: row.get("permission_warning_sent"),
            is_done: row.get("is_done")
        })
    }
}