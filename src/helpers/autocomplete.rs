use crate::data::Context;

pub async fn autocomplete_lfg_type(ctx: Context<'_>, _partial: &str) -> Vec<String> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => return Vec::new()
    };

    ctx.data().config_manager.get_types(guild_id).await
}