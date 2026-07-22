use chrono::{Duration, Utc};
use poise::serenity_prelude::{ChannelId, MessageId};
use sqlx::{Row, SqlitePool};
use crate::data::Error;
use crate::models::Session;

pub struct SessionManager {
    database: SqlitePool
}
impl SessionManager {
    pub fn new(database: SqlitePool) -> Self {
        Self {
            database
        }
    }

    pub async fn add_channel(&self,
                             session: Session
    ) -> Result<(), Error> {
        sqlx::query(
        "
            INSERT INTO sessions (
                post_id,
                guild_id,
                forum_id,
                author_id,
                announcement_channel_id,
                announcement_message_id,
                complete_message_id,
                last_activity_timestamp,
                warning_sent,
                warning_message_id,
                permission_warning_sent,
                is_done
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "
        )
            .bind(session.post_id.get() as i64)
            .bind(session.guild_id.get() as i64)
            .bind(session.forum_id.get() as i64)
            .bind(session.author_id.get() as i64)
            .bind(session.announcement_channel_id.get() as i64)
            .bind(session.announcement_message_id.get() as i64)
            .bind(session.complete_message_id.get() as i64)
            .bind(session.last_activity_timestamp)
            .bind(session.warning_sent)
            .bind(session.warning_message_id.map(|id| id.get() as i64))
            .bind(session.permission_warning_sent)
            .bind(session.is_done)
            .execute(&self.database)
            .await?;

        tracing::info!("Added session entry with channel ID {}", session.post_id);

        Ok(())
    }

    pub async fn remove_channel(&self,
                                post_id: ChannelId
    ) -> Result<bool, Error> {
        let result = sqlx::query(
        "
            DELETE FROM sessions
            WHERE post_id = ?
            "
        )
            .bind(post_id.get() as i64)
            .execute(&self.database)
            .await?;

        let removed = result.rows_affected() > 0;

        if removed {
            tracing::info!("Removed sessions entry with post ID {post_id}");
        } else {
            tracing::info!("Post ID {post_id} is not a session")
        }

        Ok(removed)
    }

    pub async fn remove_channels_by_forum(&self,
                                          forum_id: ChannelId
    ) -> Result<(), Error> {
        let result = sqlx::query(
        "
            DELETE FROM sessions
            WHERE forum_id = ?
            "
        )
            .bind(forum_id.get() as i64)
            .execute(&self.database)
            .await?;

        let rows_affected = result.rows_affected();
        let removed = rows_affected > 0;

        if removed {
            tracing::info!("Removed {rows_affected} session entries part of forum ID {forum_id}");
        } else {
            tracing::info!("Forum ID {forum_id} had no affected sessions")
        }

        Ok(())
    }

    pub async fn mark_completed(&self,
                                post_id: ChannelId
    ) -> Result<bool, Error> {
        let result = sqlx::query(
        "
            UPDATE sessions
            SET is_done = TRUE
            WHERE post_id = ?
            "
        )
            .bind(post_id.get() as i64)
            .execute(&self.database)
            .await?;

        let removed = result.rows_affected() > 0;

        if removed {
            tracing::info!("Post ID {post_id} marked as complete");
        } else {
            tracing::info!("Post ID {post_id} wasn't marked as complete. Does it exist?");
        }

        Ok(removed)
    }

    pub async fn mark_warning_sent(&self,
                                   post_id: ChannelId,
                                   warning_message_id: MessageId
    ) -> Result<(), Error> {
        sqlx::query(
        "
            UPDATE sessions
            SET
                warning_sent = TRUE,
                warning_message_id = ?
            WHERE post_id = ?
            "
        )
            .bind(warning_message_id.get() as i64)
            .bind(post_id.get() as i64)
            .execute(&self.database)
            .await?;

        tracing::debug!("Post ID {post_id} marked as warned");

        Ok(())
    }

    pub async fn mark_permission_warning_sent(&self,
                                         post_id: ChannelId
    ) -> Result<(), Error> {
        sqlx::query(
        "
            UPDATE sessions
            SET
                permission_warning_sent = TRUE
            WHERE post_id = ?
            "
        )
            .bind(post_id.get() as i64)
            .execute(&self.database)
            .await?;

        Ok(())
    }

    pub async fn update_activity(&self,
                                 post_id: ChannelId
    ) -> Result<Option<MessageId>, Error> {
        let row = sqlx::query(
        "
            SELECT warning_message_id
            FROM sessions
            WHERE post_id = ?
            "
        )
            .bind(post_id.get() as i64)
            .fetch_optional(&self.database)
            .await?;

        let warning_message_id = match row {
            Some(row) => {
                row.get::<Option<i64>, _>("warning_message_id")
                    .map(|id| MessageId::new(id as u64))
            },
            None => None
        };

        sqlx::query(
        "
            UPDATE sessions
            SET
                last_activity_timestamp = ?,
                warning_sent = FALSE,
                warning_message_id = NULL
            WHERE post_id = ?
            "
        )
            .bind(Utc::now().timestamp())
            .bind(post_id.get() as i64)
            .execute(&self.database)
            .await?;

        tracing::debug!("Updated activity for post ID {post_id}");

        Ok(warning_message_id)
    }

    pub async fn get_sessions_to_warn(&self) -> Result<Vec<Session>, Error> {
        const MINUTES_TILL_WARN: i64 = 60; // default 60

        let threshold = (Utc::now() - Duration::minutes(MINUTES_TILL_WARN)).timestamp();

        let rows = sqlx::query(
        "
            SELECT *
            FROM sessions
            WHERE
                warning_sent = FALSE
                AND is_done = FALSE
                AND last_activity_timestamp <= ?
            "
        )
            .bind(threshold)
            .fetch_all(&self.database)
            .await?;

        let mut sessions = Vec::new();

        for row in rows {
            sessions.push(Session::from_row(&row)?);
        }

        if sessions.len() > 0 {
            tracing::info!("Found {} sessions to warn", sessions.len());
        }

        Ok(sessions)
    }

    pub async fn get_sessions_to_delete(&self) -> Result<Vec<Session>, Error> {
        const MINUTES_TILL_DELETE: i64 = 90; // default 90 -> 60+30 this means 30 minutes after warn it will delete

        let threshold = (Utc::now() - Duration::minutes(MINUTES_TILL_DELETE)).timestamp();

        let rows = sqlx::query(
        "
            SELECT *
            FROM sessions
            WHERE
                last_activity_timestamp <= ?
                OR is_done = TRUE
            "
        )
            .bind(threshold)
            .fetch_all(&self.database)
            .await?;

        let mut sessions = Vec::new();

        for row in rows {
            sessions.push(Session::from_row(&row)?);
        }

        if sessions.len() > 0 {
            tracing::info!("Found {} sessions to delete", sessions.len());
        }

        Ok(sessions)
    }

    pub async fn get_all_sessions(&self) -> Result<Vec<Session>, Error> {
        let rows = sqlx::query(
        "
            SELECT *
            FROM sessions
            "
        )
            .fetch_all(&self.database)
            .await?;

        let mut sessions = Vec::new();

        for row in rows {
            sessions.push(Session::from_row(&row)?);
        }

        tracing::info!("Found {} total sessions", sessions.len());

        Ok(sessions)
    }
    
    pub async fn get_session(&self, 
                             post_id: ChannelId
    ) -> Result<Option<Session>, Error> {
        let row = sqlx::query(
        "
            SELECT *
            FROM sessions
            WHERE post_id = ?
            "
        )
            .bind(post_id.get() as i64)
            .fetch_optional(&self.database)
            .await?;

        let session = match row {
            Some(row) => {
                Some(Session::from_row(&row)?)
            },
            None => None
        };
        
        Ok(session)
    }
    
    pub async fn get_sessions_by_forum(&self,
                                       forum_id: ChannelId
    ) -> Result<Vec<Session>, Error> {
        let rows = sqlx::query(
            "
            SELECT *
            FROM sessions
            WHERE forum_id = ?
            "
        )
            .bind(forum_id.get() as i64)
            .fetch_all(&self.database)
            .await?;

        let mut sessions = Vec::new();

        for row in rows {
            sessions.push(Session::from_row(&row)?);
        }

        Ok(sessions)
    }
}