use anyhow::Result;
use dotenv::dotenv;
use interprether::{redis, transaction};
use std::time::Duration;
use web3::types::Bytes;

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
                latest_known_block_number + 1
            };

            let block = web3
                .eth()
                .block_with_txs(block_number.into())
                .await?
                .ok_or_else(|| anyhow::anyhow!("Block {} not found", block_number))?;

            let mut transactions: Vec<transaction::Transaction> = vec![];
            for tx in block.transactions.iter() {
                let _ = extract_message(tx.input.clone()).map(|message| {
                    transactions.push(transaction::Transaction {
                        message,
                        hash: format!("{:?}", tx.hash),
                        timestamp: block.timestamp.as_u64(),
                        from: tx.from.map(|from| format!("{:?}", from)),
                        to: tx.to.map(|to| format!("{:?}", to)),
                    });
                });
            }

            latest_known_block_number = block_number;

            // Save info to redis
            if !transactions.is_empty() {
                log::info!("Saving {} txs with timestamp {}", transactions.len(), block.timestamp);

                let serialized_tx = serde_json::to_string(&transactions)?;
                redis::zadd(block.timestamp.as_u64(), serialized_tx).await?;
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

pub fn extract_message(input: Bytes) -> Result<String> {
    let result = std::str::from_utf8(&input.0).map(|message| {
        // Remove NULL bytes
        message.replace(char::from(0), "").trim().to_string()
    })?;

    if result.is_empty() {
        Err(anyhow::anyhow!("Empty input data"))
    } else {
        Ok(result)
    }
}
