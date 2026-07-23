mod data;
mod models;
mod managers;
mod events;
mod housekeeping;
mod commands;
mod database;
mod helpers;
mod presence;

use std::sync::Arc;
use poise::serenity_prelude;
use poise::serenity_prelude::StatusCode;
use crate::commands::admin::{gthr_add_type, gthr_channel_config, gthr_check_permissions, gthr_remove_type};
use crate::commands::user::{help, lfg_create};
use crate::data::Data;
use crate::database::ensure_database;
use crate::events::handle_event;
use crate::helpers::cleanup::cleanup_announcement;
use crate::housekeeping::housekeeping_loop;
use crate::managers::guild_config::GuildConfigManager;
use crate::managers::session::SessionManager;
use crate::presence::presence_loop;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let token = match std::env::var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("missing DISCORD_TOKEN env variable: {e}");
            panic!("missing DISCORD_TOKEN env variable")
        }
    };

    let intents =
        serenity_prelude::GatewayIntents::GUILDS
        | serenity_prelude::GatewayIntents::GUILD_MESSAGES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                gthr_add_type(),
                gthr_channel_config(),
                gthr_remove_type(),
                gthr_check_permissions(),
                lfg_create(),
                help(),
            ],
            allowed_mentions: Some(
                serenity_prelude::CreateAllowedMentions::new()
                    .all_roles(true)
                    .all_users(true)
            ),
            event_handler: |ctx, event, framework, data| {
                Box::pin(async move {
                    handle_event(ctx, event, framework, data).await
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| Box::pin(async move {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            
            let database = ensure_database().await;

            let session_manager = Arc::new(SessionManager::new(database.clone()));
            let config_manager = Arc::new(GuildConfigManager::new(database.clone()));

            if let Err(e) = config_manager.load_type_cache().await {
                tracing::error!("Failed building type cache: {e}");
            } else {
                tracing::info!("Type cache built");
            }

            // cleaning up the database from manually deleted sessions while gather was offline
            match session_manager.get_all_sessions().await {
                Ok(sessions) => {
                    for session in sessions {
                        match session.post_id.to_channel(&ctx.http).await {
                            Ok(_) => (),
                            Err(serenity_prelude::Error::Http(http_err))
                                if http_err.status_code() == Some(StatusCode::NOT_FOUND) =>
                            {
                                cleanup_announcement(&ctx.http, &session).await;
                                session_manager.remove_channel(session.post_id).await?;
                            },
                            Err(e) => {
                                tracing::warn!("Could not verify channel ID {}: {e}", session.post_id)
                            }
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Cannot collect all sessions, initial session check failed: {e}");
                    panic!("Cannot collect all sessions, initial session check failed")
                }
            }

            tracing::info!("Finished validating sessions");

            let session_manager_housekeeping = Arc::clone(&session_manager);
            let http_housekeeping = ctx.http.clone();
            let ctx_presence = ctx.clone();
            
            // changing presence for funsies
            tokio::spawn(async move {
                presence_loop(ctx_presence).await
            });
            
            // housekeeping tasks: delete channels and send activity warnings
            tokio::spawn(async move {
                housekeeping_loop(session_manager_housekeeping, http_housekeeping).await
            });

            tracing::info!("Gather is ready");

            Ok(Data {session_manager, config_manager})
        })).build();

    let client = serenity_prelude::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await.unwrap();
}