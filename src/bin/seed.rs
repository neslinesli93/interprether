use anyhow::Result;
use dotenv::dotenv;
use interprether::{redis, transaction};
use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_IN_DAY: u64 = 86400;
const STEP: u64 = 50;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

    let max = since_the_epoch.as_secs();
    let min = max - SECONDS_IN_DAY;

    let mut counter = 0;
    let mut start = max;
    while start > min {
        let transactions = vec![transaction::Transaction {
            hash: format!("0x{}", start),
            message: format!("Message {}", start),
            timestamp: start,
            from: Some(format!("sender-0x{}", start)),
            to: Some(format!("sender-0x{}", start)),
        }];

        let serialized_tx = serde_json::to_string(&transactions)?;
        redis::zadd(start, serialized_tx).await?;

        start -= STEP;
        counter += 1;
    }

    log::info!("Inserted {} transactions", counter);

    Ok(())
}
