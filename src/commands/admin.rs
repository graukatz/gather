use crate::helpers::autocomplete::autocomplete_lfg_type;
use poise::{serenity_prelude, CreateReply};
use poise::serenity_prelude::{ChannelId, Role};
use crate::data::{Context, Error};
use crate::helpers::permissions::generate_check_permissions_reply;

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

    if let Ok(Some(reply)) = generate_check_permissions_reply(ctx).await {
        if let Some(_) = reply.content {
            ctx.send(
                CreateReply::default()
                    .content("Configuration updated, permissions have been checked")
                    .ephemeral(true)
            ).await?;
        } else {
            ctx.send(
                reply.content("Configuration updated\n## However, you should know that...")
            ).await?;
        }
    };

    Ok(())
}

/// Administrator: Check if Gather has all permissions it requires
#[poise::command(slash_command, rename="gthr-check-permissions", default_member_permissions = "ADMINISTRATOR", guild_only)]
pub async fn gthr_check_permissions(ctx: Context<'_>) -> Result<(), Error> {
    if let Ok(Some(reply)) = generate_check_permissions_reply(ctx).await {
        ctx.send(reply).await?;
    };

    Ok(())
}