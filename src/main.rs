use eth_oracle_rs::block::Transaction;
use eth_oracle_rs::redis;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::Filter;

const SECONDS_IN_DAY: u64 = 86400;

async fn get_transactions() -> Result<impl warp::Reply, warp::Rejection> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

    let max = since_the_epoch.as_secs();
    let min = max - SECONDS_IN_DAY;

    let result = redis::zrevrange_by_score(max, min).await.unwrap();

    let mut transactions: Vec<Transaction> = vec![];
    for item in result.iter() {
        let parsed: Vec<Transaction> = serde_json::from_str(item).unwrap();
        transactions.extend(parsed);
    }

    Ok(warp::reply::json(&transactions))
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "warp=info,eth_oracle_rs=info");
    env_logger::init();

    let log = warp::log("eth_oracle_rs");

    let transactions = warp::get()
        .and(warp::path("transactions"))
        .and(warp::path::end())
        .and_then(get_transactions)
        .with(log);

    warp::serve(transactions).run(([127, 0, 0, 1], 3030)).await;
}
