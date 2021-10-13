use anyhow::Result;
use dotenv::dotenv;
use interprether::{redis, transaction};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

fn rand_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let mut rng = rand::thread_rng();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let from = if rng.gen_bool(0.5) {
        Some(format!("0x{}", rand_string(16)))
    } else {
        None
    };

    let to = if rng.gen_bool(0.5) {
        Some(format!("0x{}", rand_string(16)))
    } else {
        None
    };

    let transactions = vec![transaction::Transaction {
        hash: format!("0x{}", rand_string(32)),
        message: rand_string(rng.gen_range(50..100)),
        timestamp: now.as_secs(),
        from,
        to,
    }];

    let serialized_tx = serde_json::to_string(&transactions)?;
    redis::zadd(now.as_secs(), serialized_tx).await?;

    log::info!("Inserted tx at {}", now.as_secs());

    Ok(())
}
