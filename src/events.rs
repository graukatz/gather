use poise::{serenity_prelude, FrameworkContext};
use poise::serenity_prelude::{ButtonStyle, ChannelType, CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage, FullEvent};
use crate::data::{Data, Error};
use crate::helpers::cleanup::cleanup_announcement;

pub async fn handle_event(
    ctx: &serenity_prelude::Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data
) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate{ interaction } => {
            if let serenity_prelude::Interaction::Component(component) = interaction {
                let post_id = component.channel_id;

                match data.session_manager.mark_completed(post_id).await {
                    Ok(true) => {
                        component.channel_id.edit_message(
                            &ctx.http,
                            component.message.id,
                            EditMessage::new()
                                .embed(CreateEmbed::new()
                                    .color(0xA8B31E)
                                    .title("Completed")
                                    .description("Forum post marked as completed")
                                )
                                .content(format!("Completed by <@{}>", component.user.id))
                                .button(CreateButton::new("gthr_complete")
                                    .label("Completed")
                                    .style(ButtonStyle::Secondary)
                                    .disabled(true)
                                )
                        ).await?;

                        component.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Acknowledge
                        ).await?;
                    },

                    Ok(false) => {
                        tracing::error!("Received completion button for orphaned session {post_id}");

                        component.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("Gather does not know this LFG session. Please remove manually when done")
                                    .ephemeral(true)
                            )
                        ).await?;

                        return Ok(())
                    },

                    Err(e) => {
                        tracing::error!("Failed marking session {post_id} as complete: {e}")
                    }
                }
            }
        }

        FullEvent::Message { new_message } => {
            let post_id = new_message.channel_id;

            if !new_message.author.bot {
                if let Some(warning_message_id) = data.session_manager.update_activity(post_id).await? {
                    if let Err(e) = new_message.channel_id.delete_message(&ctx.http, warning_message_id).await {
                        tracing::warn!("Failed to delete warning message: {e}");
                    }
                }
            }
        }

        FullEvent::ThreadDelete { thread, ..} => {
            match data.session_manager.get_session(thread.id).await {
                Ok(Some(session)) => {
                    cleanup_announcement(&ctx.http, &session).await;

                    match data.session_manager.remove_channel(thread.id).await {
                        Ok(_) => (),
                        Err(e) => {
                            tracing::warn!("Failed to delete channel ID {}: {e}", thread.id);
                        }
                    }
                },

                Ok(None) => (),

                Err(e) => {
                    tracing::warn!("Failed retrieving deleted forum post {}: {e}", thread.id)
                }
            }
        }

        FullEvent::ChannelDelete { channel, ..} => {
            if channel.kind == ChannelType::Forum {
                match data.session_manager.get_sessions_by_forum(channel.id).await {
                    Ok(sessions) => {
                        if !sessions.is_empty() {
                            for session in sessions {
                                cleanup_announcement(&ctx.http, &session).await;
                            }

                            if let Err(e) = data.session_manager.remove_channels_by_forum(channel.id).await {
                                tracing::warn!("Failed to remove sessions for deleted forum {}: {e}", channel.id)
                            }
                        }
                    },

                    Err(e) => {
                        tracing::warn!("Failed getting sessions for deleted forum {}: {e}", channel.id)
                    }
                }
            }
        }

        _ => {}
    }

    Ok(())
}