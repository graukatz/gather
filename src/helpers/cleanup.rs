use std::sync::Arc;
use poise::serenity_prelude::{CreateMessage, Http};
use crate::managers::session::SessionManager;
use crate::models::Session;

// delete forum post
pub async fn cleanup_post(
    http: &Http,
    session: &Session,
    session_manager: Arc<SessionManager>
) {
    match session.post_id.delete(http).await {
        Ok(_) => (),

        Err(e) => {
            if e.to_string() == "Missing Permissions" && !session.permission_warning_sent {
                println!("missing permissions check");
                match session.post_id.send_message(http, CreateMessage::new().content("Gather could not delete this post because it is missing permissions. Please ask an Administrator to check my permissions.")).await {
                    Ok(_) => {
                        if let Err(e) = session_manager.mark_permission_warning_sent(session.post_id).await {
                            tracing::error!("Failed marking permission warning sent for post {}: {e}", session.post_id);
                        };
                    }

                    Err(e) => {
                        tracing::warn!("Failed sending message: {e}");
                    }
                }
            }
        }
    }
}

// delete announcement message
pub async fn cleanup_announcement(
    http: &Http,
    session: &Session
) {
    if let Err(e) = session.announcement_channel_id.delete_message(http, session.announcement_message_id).await {
        tracing::warn!("Failed cleaning up announcement {}: {e}", session.announcement_message_id);
    };
}