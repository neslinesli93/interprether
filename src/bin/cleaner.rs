use anyhow::Result;
use dotenv::dotenv;
use interprether::redis;
use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_IN_DAY: u64 = 86400;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

    let max = since_the_epoch.as_secs() - SECONDS_IN_DAY;

    let cleaned_values = redis::zremrange_by_score(max).await?;
    log::info!("Removed {} values from set", cleaned_values);

    Ok(())
}
