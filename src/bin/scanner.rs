use std::time::Duration;

const GETH_URL: &str = "ws://localhost:8546";

#[tokio::main]
async fn main() -> web3::Result<()> {
    let transport = web3::transports::WebSocket::new(GETH_URL).await?;
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

            for tx in block.transactions.iter() {
                let input = tx.input.clone();

                match std::str::from_utf8(&input.0) {
                    Ok("") => (),
                    Ok(message) => println!("Tx {:?} has text: {}", tx.hash, message),
                    _ => (),
                }
            }

            latest_known_block_number = block_number;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
