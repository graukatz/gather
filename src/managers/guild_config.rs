use std::collections::HashMap;
use poise::serenity_prelude::{ChannelId, GuildId, RoleId};
use sqlx::{Row, SqlitePool};
use tokio::sync::RwLock;
use crate::data::Error;

pub struct GuildConfigManager {
    database: SqlitePool,
    types_cache: RwLock<HashMap<GuildId, Vec<String>>>
}
impl GuildConfigManager {
    pub fn new(database: SqlitePool) -> Self {
        Self {
            database,
            types_cache: RwLock::new(HashMap::new())
        }
    }

    pub async fn set_config(&self,
                            guild_id: GuildId,
                            forum_id: ChannelId,
                            lfg_command_channel_id: ChannelId
    ) -> Result<(), Error> {
        let result = sqlx::query(
            "
                INSERT INTO guild_config (guild_id, forum_id, lfg_command_channel_id)
                VALUES (?, ?, ?)
                ON CONFLICT(guild_id)
                DO UPDATE SET
                    forum_id = excluded.forum_id,
                    lfg_command_channel_id = excluded.lfg_command_channel_id
                "
        )
            .bind(guild_id.get() as i64)
            .bind(forum_id.get() as i64)
            .bind(lfg_command_channel_id.get() as i64)
            .execute(&self.database)
            .await?;

        let updated = result.rows_affected() > 0;

        if updated {
            tracing::info!("Guild {guild_id} updated forum ID to {forum_id} and lfg command channel ID to {lfg_command_channel_id}");
        }

        Ok(())
    }

    pub async fn add_type(&self,
                          guild_id: GuildId,
                          name: String,
                          role_id: RoleId
    ) -> Result<(), Error> {
        let result = sqlx::query(
            "
                INSERT INTO lfg_types (guild_id, name, role_id)
                VALUES (?, ?, ?)
                ON CONFLICT(guild_id, name)
                DO UPDATE SET
                   role_id = excluded.role_id
                "
        )
            .bind(guild_id.get() as i64)
            .bind(name.clone())
            .bind(role_id.get() as i64)
            .execute(&self.database)
            .await?;

        if result.rows_affected() > 0 {
            tracing::info!("Guild {guild_id} added/updated LFG type with name {name}");

            let mut cache = self.types_cache.write().await;

            cache
                .entry(guild_id)
                .or_insert_with(Vec::new)
                .push(name);
        }

        Ok(())
    }

    pub async fn remove_type(&self,
                             guild_id: GuildId,
                             name: String
    ) -> Result<bool, Error> {
        let result = sqlx::query(
            "
                DELETE FROM lfg_types
                WHERE guild_id = ?
                AND name = ?
                "
        )
            .bind(guild_id.get() as i64)
            .bind(name.clone())
            .execute(&self.database)
            .await?;

        let removed = result.rows_affected() > 0;

        if removed {
            tracing::info!("Guild {guild_id} removed LFG type with name {name}");

            let mut cache = self.types_cache.write().await;

            if let Some(types) = cache.get_mut(&guild_id) {
                types.retain(|x| x != &name)
            }
        }

        Ok(removed)
    }

    pub async fn get_role_from_name(&self,
                                    guild_id: GuildId,
                                    name: String
    ) -> Option<RoleId> {
        tracing::debug!("Getting role from LFG type {name} for Guild {guild_id}");

        let row = sqlx::query(
            "
                SELECT role_id
                FROM lfg_types
                WHERE guild_id = ?
                AND name = ?
            "
        )
            .bind(guild_id.get() as i64)
            .bind(name)
            .fetch_optional(&self.database)
            .await
            .ok()??;

        let role_id_raw: i64 = row.get("role_id");

        Some(RoleId::new(role_id_raw as u64))
    }

    pub async fn get_forum_id(&self,
                              guild_id: GuildId
    ) -> Option<ChannelId> {
        tracing::debug!("Getting forum ID from Guild {guild_id}");

        let row = sqlx::query(
            "
                SELECT forum_id
                FROM guild_config
                WHERE guild_id = ?
            "
        )
            .bind(guild_id.get() as i64)
            .fetch_optional(&self.database)
            .await
            .ok()??;

        let forum_id_raw: i64 = row.get("forum_id");

        Some(ChannelId::new(forum_id_raw as u64))
    }

    pub async fn load_type_cache(&self) -> Result<(), Error> {
        let rows = sqlx::query(
        "
            SELECT guild_id, name
            FROM lfg_types
            "
        )
            .fetch_all(&self.database)
            .await?;

        let mut cache = self.types_cache.write().await;

        cache.clear();

        for row in rows {
            let guild_id = GuildId::new(row.get("guild_id"));
            let name: String = row.get("name");

            cache
                .entry(guild_id)
                .or_insert_with(Vec::new)
                .push(name);
        }

        Ok(())
    }

    pub async fn get_types(&self,
                           guild_id: GuildId
    ) -> Vec<String> {
        tracing::debug!("Getting all types from Guild {guild_id}");

        let cache = self.types_cache.read().await;

        cache
            .get(&guild_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn get_lfg_command_channel(&self,
                              guild_id: GuildId
    ) -> Option<ChannelId> {
        tracing::debug!("Getting lfg command channel ID from Guild {guild_id}");

        let row = sqlx::query(
            "
                SELECT lfg_command_channel_id
                FROM guild_config
                WHERE guild_id = ?
            "
        )
            .bind(guild_id.get() as i64)
            .fetch_optional(&self.database)
            .await
            .ok()??;

        let get_lfg_command_channel_raw: i64 = row.get("lfg_command_channel_id");

        Some(ChannelId::new(get_lfg_command_channel_raw as u64))
    }
}