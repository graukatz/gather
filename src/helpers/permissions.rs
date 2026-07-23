use poise::CreateReply;
use poise::serenity_prelude::{ChannelId, CreateEmbed, CreateEmbedFooter, GuildChannel};
use crate::data::{Context, Error};

pub async fn generate_check_permissions_reply(ctx: Context<'_>) -> Result<Option<CreateReply>, Error> {
    let guild_id = ctx.guild_id().ok_or("Command can only be used in a server")?;
    let guild = guild_id.to_partial_guild(&ctx.http()).await?;

    let bot_id = ctx.framework().bot_id;
    let bot_member = guild.member(&ctx.http(), bot_id).await?;

    let forum_id_opt = ctx.data().config_manager.get_forum_id(guild_id).await;
    let forum = match get_guild_channel(ctx, forum_id_opt).await? {
        Some(guild_channel) => guild_channel,
        None => return Ok(None)
    };

    let lfg_command_channel_id_opt = ctx.data().config_manager.get_lfg_command_channel(guild_id).await;
    let lfg_command_channel = match get_guild_channel(ctx, lfg_command_channel_id_opt).await? {
        Some(guild_channel) => guild_channel,
        None => return Ok(None)
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

    if forum_missing.len() > 0 || lfg_command_channel_missing.len() > 0 {
        let mut embed = CreateEmbed::new()
            .color(0xA8B31E)
            .title("Gather is missing permissions!")
            .footer(
                CreateEmbedFooter::new("Try re-adding Gather to your server or check the channel overrides")
            );

        if forum_missing.len() > 0 {
            embed = embed.field("LFG Forum Channel", format!("Link: {}\n{}", forum, forum_missing.join("\n")), true);
        }

        if lfg_command_channel_missing.len() > 0 {
            embed = embed.field("LFG Command Channel", format!("Link: {}\n{}", lfg_command_channel, lfg_command_channel_missing.join("\n")), true);
        }

        let reply = CreateReply::default()
            .ephemeral(true)
            .embed(embed);

        return Ok(Some(reply))
    }

    Ok(Some(CreateReply::default()
        .ephemeral(true)
        .content("Gather has all necessary permissions")))
}

async fn get_guild_channel(ctx: Context<'_>,
                           channel_id_opt: Option<ChannelId>
) -> Result<Option<GuildChannel>, Error> {
    let channel_id = match channel_id_opt {
        Some(channel_id_opt) => channel_id_opt,

        None => {
            ctx.send(
                CreateReply::default()
                    .content("This server has no channels configured. Configure channels using '/gthr-channel-config'")
                    .ephemeral(true)
            ).await?;

            return Ok(None)
        }
    };

    match channel_id.to_channel(ctx.http()).await {
        Ok(channel) => match channel.guild() {
            Some(guild_channel) => Ok(Some(guild_channel)),
            None => {
                ctx.send(
                    CreateReply::default()
                        .content("This server has no channels configured. Configure channels using '/gthr-channel-config'")
                        .ephemeral(true)
                ).await?;

                Ok(None)
            }
        }

        Err(e) => {
            ctx.send(
                CreateReply::default()
                    .content(format!("{e}. Check the 'View Channel' permission on your configured channels"))
                    .ephemeral(true)
            ).await?;

            Ok(None)
        }
    }
}