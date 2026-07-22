use std::sync::Arc;
use poise::serenity_prelude;
use poise::serenity_prelude::{Http, Mentionable};
use crate::helpers::cleanup::cleanup_post;
use crate::managers::session::SessionManager;
use crate::models::Session;

pub async fn housekeeping_loop(
    session_manager_task: Arc<SessionManager>,
    http_task: Arc<Http>
) {
    loop {
        const CYCLE_TIME_SECONDS: u64 = 60; // default 60
        tokio::time::sleep(core::time::Duration::from_secs(CYCLE_TIME_SECONDS)).await;
        tracing::debug!("Running housekeeping cycle");

        match session_manager_task.get_sessions_to_delete().await {
            Ok(sessions) => {
                for session in sessions {
                    cleanup_post(&http_task, &session, session_manager_task.clone()).await;
                }
            },
            Err(e) => {
                tracing::error!("Failed to get sessions to delete: {e}");
            }
        };

        match session_manager_task.get_sessions_to_warn().await {
            Ok(sessions) => {
                for session in sessions {
                    warn_session(&http_task, &session_manager_task, session).await
                }
            },
            Err(e) => {
                tracing::error!("Failed to get sessions to warn: {e}");
            }
        };

        tracing::debug!("Finished housekeeping cycle");
    }
}

async fn warn_session(
    http: &Http,
    manager: &SessionManager,
    session: Session
) {
    let author_mention = session.author_id.mention();
    let complete_message_link = session.complete_message_id.link(session.post_id, Option::from(session.guild_id));

    match session.post_id.send_message(&http, serenity_prelude::CreateMessage::new()
        .content(format!("Hey {author_mention}\n\
        This post has been inactive for an hour. If I don't see any further activity in this forum post within the next 30 minutes, it will be deleted.\n\
        Please mark this post as completed if you are done here: {complete_message_link}\n\n\
        -# Send a message to mark this post as active"))
    ).await {
        Ok(warn_message) => {
            match manager.mark_warning_sent(session.post_id, warn_message.id).await {
                Ok(_) => (),

                Err(e) => {
                    tracing::error!("Failed marking warning sent for post {}: {e}", session.post_id);
                }
            }
        }

        Err(e) => {
            tracing::warn!("Failed sending message: {e}");
        }
    };
}