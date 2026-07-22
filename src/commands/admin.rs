use crate::helpers::autocomplete::autocomplete_lfg_type;
use poise::{serenity_prelude, CreateReply};
use poise::serenity_prelude::{ChannelId, CreateEmbed, CreateEmbedFooter, Role};
use crate::data::{Context, Error};

/// Administrator: Create or update a specified type
#[poise::command(slash_command, rename="gthr-add-type", default_member_permissions = "ADMINISTRATOR", guild_only)]
pub async fn gthr_add_type(ctx: Context<'_>,
                       #[description = "Name of the type (e.g. business-sales)"]
                       name: String,
                       #[description = "Role which is going to be notified for this type"]
                       role: Role
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command can only be used in a server")?;

    if let Err(e) = ctx.data()
        .config_manager
        .add_type(guild_id, name, role.id)
        .await
    {
        ctx.send(
            CreateReply::default()
                .content("Failed to add type. Please try again later")
                .ephemeral(true)
        ).await?;

        tracing::error!("Failed to add LFG type for Guild {guild_id}: {e}");

        return Ok(())
    }

    ctx.send(
        CreateReply::default()
            .content("Configuration updated")
            .ephemeral(true)
    ).await?;

    Ok(())
}

/// Administrator: Remove a specified type
#[poise::command(slash_command, rename="gthr-remove-type", default_member_permissions = "ADMINISTRATOR", guild_only)]
pub async fn gthr_remove_type(ctx: Context<'_>,
                          #[description = "Name of the type to delete"]
                          #[autocomplete = "autocomplete_lfg_type"]
                          name: String
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command can only be used in a server")?;

    let removed = ctx.data()
        .config_manager
        .remove_type(guild_id, name.clone())
        .await;

    let reply = match removed {
        Ok(bool) => {
            if bool {
                format!("Removed type '{name}'")
            } else {
                format!("No type named '{name}' exists")
            }
        },
        Err(e) => {
            tracing::error!("Failed to remove LFG type for Guild {guild_id}: {e}");
            "Failed to remove type. Please try again later".to_string()
        }
    };

    ctx.send(
        CreateReply::default()
            .content(reply)
            .ephemeral(true)
    ).await?;

    Ok(())
}

/// Administrator: Configure Channels for Gather
#[poise::command(slash_command, rename="gthr-channel-config", default_member_permissions = "ADMINISTRATOR", guild_only)]
pub async fn gthr_channel_config(ctx: Context<'_>,
                            #[rename = "forum"]
                            #[description = "Forum where new forum posts should show up"]
                            forum_id: ChannelId,
                            #[rename = "command"]
                            #[description = "Channel where /lfg-create must be used"]
                            lfg_command_channel_id: ChannelId
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command can only be used in a server")?;

    let forum_channel = forum_id.to_channel(&ctx.http()).await?;

    match forum_channel {
        serenity_prelude::Channel::Guild(guild_channel) => {
            if guild_channel.kind != serenity_prelude::ChannelType::Forum {
                ctx.send(
                    CreateReply::default()
                        .content("Forum must be a forum channel")
                        .ephemeral(true)
                ).await?;

                return Ok(());
            }
        }
        _ => {
            ctx.send(
                CreateReply::default()
                    .content("Forum channel must be a server channel")
                    .ephemeral(true)
            ).await?;

            return Ok(());
        }
    };

    let lfg_command_channel = lfg_command_channel_id.to_channel(&ctx.http()).await?;

    match lfg_command_channel {
        serenity_prelude::Channel::Guild(guild_channel) => {
            match guild_channel.kind {
                serenity_prelude::ChannelType::Text => {}
                _ => {
                    ctx.send(
                        CreateReply::default()
                            .content("Command channel must be a text channel")
                            .ephemeral(true)
                    ).await?;

                    return Ok(());
                }
            }
        },
        _ => {
            ctx.send(
                CreateReply::default()
                    .content("Command channel must be a server channel")
                    .ephemeral(true)
            ).await?;

            return Ok(());
        }
    };

    ctx.data()
        .config_manager
        .set_config(guild_id, forum_id, lfg_command_channel_id)
        .await?;

    ctx.send(
        CreateReply::default()
            .content("Configuration updated")
            .ephemeral(true)
    ).await?;

    Ok(())
}

/// Administrator: Check if Gather has all permissions it requires
#[poise::command(slash_command, rename="gthr-check-permissions", default_member_permissions = "ADMINISTRATOR", guild_only)]
pub async fn gthr_check_permissions(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Command can only be used in a server")?;
    let guild = guild_id.to_partial_guild(&ctx.http()).await?;

    let bot_id = ctx.framework().bot_id;
    let bot_member = guild.member(&ctx.http(), bot_id).await?;

    //TODO duplicate code
    let forum_id = match ctx.data().config_manager.get_forum_id(guild_id).await {
        Some(forum_id) => forum_id,

        None => {
            ctx.send(
                CreateReply::default()
                    .content("This server has no channels configured. Configure channels using '/gthr-channel-config'")
                    .ephemeral(true)
            ).await?;

            return Ok(())
        }
    };
    let forum = match forum_id.to_channel(ctx.http()).await {
        Ok(forum) => {
            match forum.guild() {
                Some(guild_channel) => guild_channel,
                None => {
                    ctx.send(
                        CreateReply::default()
                            .content("This server has no channels configured. Configure channels using '/gthr-channel-config'")
                            .ephemeral(true)
                    ).await?;

                    return Ok(())
                }
            }
        },

        Err(e) => {
            ctx.send(
                CreateReply::default()
                    .content(format!("{e} | Configure channels using '/gthr-channel-config'"))
                    .ephemeral(true)
            ).await?;

            return Ok(())
        }
    };

    let lfg_command_channel_id = match ctx.data().config_manager.get_lfg_command_channel(guild_id).await {
        Some(lfg_command_channel_id) => lfg_command_channel_id,

        None => {
            ctx.send(
                CreateReply::default()
                    .content("This server has no channels configured. Configure channels using '/gthr-channel-config'")
                    .ephemeral(true)
            ).await?;

            return Ok(())
        }
    };
    let lfg_command_channel = match lfg_command_channel_id.to_channel(ctx.http()).await {
        Ok(command_channel) => {
            match command_channel.guild() {
                Some(guild_channel) => guild_channel,
                None => {
                    ctx.send(
                        CreateReply::default()
                            .content("This server has no channels configured. Configure channels using '/gthr-channel-config'")
                            .ephemeral(true)
                    ).await?;

                    return Ok(())
                }
            }
        },

        Err(e) => {
            ctx.send(
                CreateReply::default()
                    .content(format!("{e} | Configure channels using '/gthr-channel-config'"))
                    .ephemeral(true)
            ).await?;

            return Ok(())
        }
    };

    let forum_permissions = guild.user_permissions_in(&forum, &bot_member);
    let lfg_command_channel_permissions = guild.user_permissions_in(&lfg_command_channel, &bot_member);

    let mut forum_missing = Vec::new();
    let mut lfg_command_channel_missing = Vec::new();

    if !forum_permissions.view_channel() {
        forum_missing.push("- View Channel")
    }
    if !forum_permissions.send_messages() || !forum_permissions.send_messages_in_threads() {
        forum_missing.push("- Create Posts / Send Messages")
    }
    if !forum_permissions.create_public_threads() {
        forum_missing.push("- Create Public Threads")
    }
    if !forum_permissions.manage_channels() {
        forum_missing.push("- Manage Channels")
    }
    if !forum_permissions.read_message_history() {
        forum_missing.push("- Read Message History")
    }

    if !lfg_command_channel_permissions.view_channel() {
        lfg_command_channel_missing.push("- View Channel")
    }
    if !lfg_command_channel_permissions.send_messages() {
        lfg_command_channel_missing.push("- Send Messages")
    }
    if !lfg_command_channel_permissions.manage_messages() {
        lfg_command_channel_missing.push("- Manage Messages")
    }
    if !lfg_command_channel_permissions.mention_everyone() {
        lfg_command_channel_missing.push("- Mention @everyone, @here and All Roles")
    }
    if !lfg_command_channel_permissions.read_message_history() {
        lfg_command_channel_missing.push("- Read Message History")
    }

    let mut reply = CreateReply::default()
        .ephemeral(true);

    if forum_missing.len() == 0 && lfg_command_channel_missing.len() == 0 {
        reply = reply.content("Gather has all necessary permissions");
    } else {
        let mut embed = CreateEmbed::new()
            .color(0xA8B31E)
            .title("Gather is missing permissions!")
            .footer(
                CreateEmbedFooter::new("Try re-adding Gather to your server or check the channel overrides")
            );

        if forum_missing.len() > 0 {
            embed = embed.field("LFG Forum Channel", format!("{}\n{}", forum, forum_missing.join("\n")), true);
        }

        if lfg_command_channel_missing.len() > 0 {
            embed = embed.field("LFG Command Channel", format!("Link: {}\n{}", lfg_command_channel, lfg_command_channel_missing.join("\n")), true);
        }

        reply = reply.embed(embed)
    }

    ctx.send(reply).await?;

    Ok(())
}