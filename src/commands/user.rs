use crate::helpers::autocomplete::autocomplete_lfg_type;
use poise::{serenity_prelude, CreateReply};
use poise::serenity_prelude::{ButtonStyle, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage, GetMessages, Mentionable};
use crate::data::{Context, Error};
use crate::helpers::cleanup::{cleanup_announcement, cleanup_post};
use crate::models::Session;

/// Create a new LFG session
#[poise::command(slash_command, rename="lfg-create", guild_only, member_cooldown = 10)]
pub async fn lfg_create(ctx: Context<'_>,
                    #[rename="type"]
                    #[description = "The LFG type/name"]
                    #[autocomplete = "autocomplete_lfg_type"]
                    type_name: String,
                    #[description = "Number of players you are searching for ('3-4', '2', 'about 3' are all valid options)"]
                    players: String,
                    #[rename="message"]
                    #[description = "Optional message to provide more information in the announcement"]
                    optional_message: Option<String>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command can only be used in a server")?;
    let announcement_channel_id = ctx.channel_id();

    match ctx.data().config_manager.get_lfg_command_channel(guild_id).await {
        Some(channelid) => {
            match channelid.to_channel(&ctx.http()).await {
                Ok(_) => {
                    if channelid != announcement_channel_id {
                        ctx.send(
                            CreateReply::default()
                                .content(format!("Command can only be used inside {}", channelid.mention()))
                                .ephemeral(true)
                        ).await?;

                        return Ok(())
                    }
                }

                Err(_) => {
                    ctx.send(
                    CreateReply::default()
                    .content("This server has no channels configured. Ask an administrator to configure channels using '/gthr-channel-config'")
                    .ephemeral(true)
                    ).await?;

                    return Ok(())
                }
            }
        }

        None => {
            ctx.send(
                CreateReply::default()
                    .content("This server has no channels configured. Ask an administrator to configure channels using '/gthr-channel-config'")
                    .ephemeral(true)
            ).await?;

            return Ok(())
        }
    };

    let forum_id = match ctx.data()
        .config_manager
        .get_forum_id(guild_id)
        .await
    {
        Some(forum_id) => {
            if let Err(_) = forum_id.to_channel(&ctx.http()).await {
                ctx.send(
                    CreateReply::default()
                        .content("This server has no channels configured. Ask an administrator to configure channels using '/gthr-channel-config'")
                        .ephemeral(true)
                ).await?;

                return Ok(())
            } else {
                forum_id
            }
        }

        None => {
            ctx.send(
                CreateReply::default()
                    .content("This server has no channels configured. Ask an administrator to configure channels using '/gthr-channel-config'")
                    .ephemeral(true)
            ).await?;

            return Ok(())
        }
    };

    let role_id = match ctx.data()
        .config_manager
        .get_role_from_name(guild_id, type_name.clone())
        .await
    {
        Some(role_id) => role_id,

        None => {
            ctx.send(
                CreateReply::default()
                    .content("Unknown LFG type, check autocomplete for available options")
                    .ephemeral(true)
            ).await?;

            return Ok(())
        }
    };

    let author_name = &ctx.author().name;
    let author_id = ctx.author().id;

    // create forum post
    let post_channel = forum_id.create_forum_post(
        ctx.serenity_context(),
        serenity_prelude::CreateForumPost::new(
            format!("{} {}", type_name, author_name),
            CreateMessage::default()
                .embed(CreateEmbed::new()
                    .color(0xA8B31E)
                    .title("Welcome!")
                    .description("When you are done, please use the button below to mark this forum post as completed")
                )
                .button(CreateButton::new("gthr_complete")
                    .label("Mark as complete ☑️")
                    .style(ButtonStyle::Secondary))
        )
    ).await?;

    let post_id = post_channel.id;
    let complete_message_id = post_channel
        .messages(&ctx.http(), GetMessages::new().limit(1))
        .await?
        .first()
        .ok_or("Forum post has no initial message")?.id;

    // creating and sending announcement to according role
    let mut announcement_embed = CreateEmbed::new()
        .color(0xA8B31E)
        .title(format!("🔍 Looking For Group: {type_name}"))
        .field("Host", format!("<@{author_id}>"), false)
        .field("Players needed", &players, false)
        .field("Forum Post", format!("<#{post_id}>"), false);

    match optional_message {
        Some(message) => {
            announcement_embed = announcement_embed.footer(
                CreateEmbedFooter::new(format!("More info: {message}"))
            )
        }
        None => ()
    };

    let announcement = ctx.send(CreateReply::default()
        .content(format!("<@&{role_id}>"))
        .embed(announcement_embed)).await?;

    let announcement_message_id = announcement.message().await?.id;

    // put created session in database via manager if forum post still exists
    let session = Session::new(
        post_id,
        guild_id,
        forum_id,
        author_id,
        announcement_channel_id,
        announcement_message_id,
        complete_message_id
    );
    
    // check if forum post still exists
    if let Err(e) = post_id.to_channel(&ctx.http()).await {
        tracing::warn!("Configured forum {forum_id} is unavailable: {e}");

        ctx.send(
            CreateReply::default()
                .content("The LFG forum was deleted while creating your post. Ask an administrator to configure one using '/gthr-channel-config'")
                .ephemeral(true)
        ).await?;
        
        cleanup_announcement(&ctx.http(), &session).await;

        return Ok(())
    }
    // insert, if db errors occur: cleanup
    if let Err(e) = ctx.data().session_manager.add_channel(session.clone()).await {
        tracing::error!("Failed storing session {post_id} in database: {e}");

        cleanup_announcement(&ctx.http(), &session).await;
        cleanup_post(&ctx.http(), &session, ctx.data().session_manager.clone()).await;

        ctx.send(
            CreateReply::default()
                .content("Failed to create session. Please try again later")
                .ephemeral(true)
        ).await?;

        return Ok(())
    }

    Ok(())
}

/// List available commands and receive information about Gather
#[poise::command(slash_command, rename="help", guild_only)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .embed(CreateEmbed::new()
                .color(0xA8B31E)
                .title("Available commands")
                .description("Gather consists of slash commands:")
                .field("User",
                       "`/help`\n`/lfg-create`", true)
                .field("Admin",
                       "`/gthr-add-type`\n`/gthr-remove-type`\n`/gthr-channel-config`\n`/gthr-check-permissions`", true)
                .field("Server Setup",
                       "Add a type and run the channel config to get started. It is highly recommended to run `/gthr-check-permissions` after each channel config change to ensure full functionality", false)
                .field("Information", "Forum posts are warned as inactive after one hour.\n\
                After a warning, forum posts with no further activity are deleted in 30 minutes.", false)
                .footer(CreateEmbedFooter::new("/help"))
            )
            .ephemeral(true)
    ).await?;

    Ok(())
}