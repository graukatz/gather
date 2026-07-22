use std::sync::Arc;
use crate::managers::guild_config::GuildConfigManager;
use crate::managers::session::SessionManager;

pub struct Data {
    pub session_manager: Arc<SessionManager>,
    pub config_manager: Arc<GuildConfigManager>
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Context<'a> = poise::Context<'a, Data, Error>;