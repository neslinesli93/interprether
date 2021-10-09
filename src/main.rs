use dotenv::dotenv;
use eth_oracle_rs::block::Transaction;
use eth_oracle_rs::redis;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::Filter;

const SECONDS_IN_DAY: u64 = 86400;

#[derive(Debug)]
struct ServerError;

impl warp::reject::Reject for ServerError {}

// Query params for /transactions
#[derive(Debug, Deserialize)]
pub struct TransactionsQueryParams {
    pub after: Option<u64>,
    pub limit: Option<usize>,
}

async fn get_data(params: TransactionsQueryParams) -> anyhow::Result<Vec<Transaction>> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time awent backwards");
    let max = since_the_epoch.as_secs();

    let mut min = max - SECONDS_IN_DAY;
    if let Some(a) = params.after {
        min = a + 1;
    }

    let result = redis::zrevrange_by_score(max, min).await?;

    let mut transactions: Vec<Transaction> = vec![];
    for item in result.iter() {
        let parsed: Vec<Transaction> = serde_json::from_str(item)?;
        transactions.extend(parsed);
    }

    match params.limit {
        Some(l) => Ok(transactions[0..l].to_vec()),
        None => Ok(transactions),
    }
}

async fn get_transactions(params: TransactionsQueryParams) -> anyhow::Result<impl warp::Reply, warp::Rejection> {
    match get_data(params).await {
        Ok(transactions) => Ok(warp::reply::json(&transactions)),
        Err(error) => {
            log::error!("Error while fetching txs: {:?}", error);
            Err(warp::reject::custom(ServerError))
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "warp=info,eth_oracle_rs=info");
    env_logger::init();

    let log = warp::log("eth_oracle_rs");

    let origin = std::env::var("ORIGIN").expect("ORIGIN must be set");
    let cors = warp::cors().allow_origin(origin.as_str());

    let transactions = warp::get()
        .and(warp::path("transactions"))
        .and(warp::path::end())
        .and(warp::query::<TransactionsQueryParams>())
        .and_then(get_transactions)
        .with(log)
        .with(cors);

    warp::serve(transactions).run(([0, 0, 0, 0], 3030)).await;
}
