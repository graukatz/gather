use poise::serenity_prelude::{ActivityData, Context};

pub async fn presence_loop(ctx: Context) {
    const CYCLE_TIME_SECONDS: u64 = 30; // default 30

    let activities = [
        "observing... | /help",
        "managing... | /help",
        "finding... | /help",
        "organising... | /help",
        "assembling... | /help",
        "searching... | /help",
        "assisting... | /help"
    ];

    let mut index = 0;

    loop {
        ctx.set_activity(Some(ActivityData::custom(activities[index])));

        index = (index + 1) % activities.len();

        tokio::time::sleep(core::time::Duration::from_secs(CYCLE_TIME_SECONDS)).await;
    }
}