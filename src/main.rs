use eth_oracle_rs::block::Transaction;
use eth_oracle_rs::redis;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::Filter;

const SECONDS_IN_DAY: u64 = 86400;

#[derive(Debug)]
struct ServerError;

impl warp::reject::Reject for ServerError {}

async fn get_data() -> anyhow::Result<Vec<Transaction>> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

    let max = since_the_epoch.as_secs();
    let min = max - SECONDS_IN_DAY;

    let result = redis::zrevrange_by_score(max, min).await?;

    let mut transactions: Vec<Transaction> = vec![];
    for item in result.iter() {
        let parsed: Vec<Transaction> = serde_json::from_str(item)?;
        transactions.extend(parsed);
    }

    Ok(transactions)
}

async fn get_transactions() -> anyhow::Result<impl warp::Reply, warp::Rejection> {
    match get_data().await {
        Ok(transactions) => Ok(warp::reply::json(&transactions)),
        Err(error) => {
            log::error!("Error while fetching txs: {:?}", error);
            Err(warp::reject::custom(ServerError))
        }
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "warp=info,eth_oracle_rs=info");
    env_logger::init();

    let log = warp::log("eth_oracle_rs");

    let cors = warp::cors().allow_origin("http://localhost:8080");

    let transactions = warp::get()
        .and(warp::path("transactions"))
        .and(warp::path::end())
        .and_then(get_transactions)
        .with(log)
        .with(cors);

    warp::serve(transactions).run(([127, 0, 0, 1], 3030)).await;
}
