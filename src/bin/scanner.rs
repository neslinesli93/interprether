use anyhow::Result;
use dotenv::dotenv;
use eth_oracle_rs::{block, redis};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    log::info!("Scanner started");

    let geth_url = std::env::var("WEB3_PROVIDER_URL").expect("WEB3_PROVIDER_URL must be set");
    let transport = web3::transports::Http::new(&geth_url)?;
    let web3 = web3::Web3::new(transport);

    let mut latest_known_block_number = web3::types::U64::from(0);

    loop {
        let current_block_number = web3.eth().block_number().await?;

        while latest_known_block_number.is_zero() || current_block_number > latest_known_block_number {
            let block_number = if latest_known_block_number.is_zero() {
                current_block_number
            } else {
                (latest_known_block_number + 1).into()
            };

            let block = web3.eth().block_with_txs(block_number.into()).await?.unwrap();

            let mut transactions: Vec<block::Transaction> = vec![];
            for tx in block.transactions.iter() {
                let input = tx.input.clone();

                let _ = std::str::from_utf8(&input.0).map(|message| {
                    // Remove NULL bytes
                    let cleaned_message = message.replace(char::from(0), "");
                    if !cleaned_message.is_empty() {
                        transactions.push(block::Transaction::new(
                            tx.hash,
                            cleaned_message,
                            block.timestamp.as_u64(),
                        ));
                    }
                });
            }

            latest_known_block_number = block_number;

            // Save info to redis
            if transactions.len() > 0 {
                log::info!("Saving {} txs with timestamp {}", transactions.len(), block.timestamp);

                let serialized_tx = serde_json::to_string(&transactions)?;
                redis::zadd(block.timestamp.as_u64(), serialized_tx).await?;
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
