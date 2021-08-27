use anyhow::Result;
use deadpool_redis::redis::cmd;
use deadpool_redis::{Config, Pool};
use once_cell::sync::Lazy;
use std::sync::Arc;

const REDIS_URL: &str = "redis://0.0.0.0:6901";
const TX_SORTED_SET: &str = "tx_set";

static POOL: Lazy<Arc<Pool>> = Lazy::new(|| {
    let cfg = Config::from_url(REDIS_URL);
    let pool = cfg.create_pool().unwrap();

    Arc::new(pool)
});

pub async fn zadd(score: u64, value: String) -> Result<()> {
    let mut conn = POOL.get().await?;

    let _ = cmd("ZADD")
        .arg(&[TX_SORTED_SET.to_string(), score.to_string(), value])
        .query_async::<_, ()>(&mut conn)
        .await?;

    Ok(())
}

pub async fn zrevrange_by_score(max: u64, min: u64) -> Result<Vec<String>> {
    let mut conn = POOL.get().await?;

    let value: Vec<String> = cmd("ZREVRANGEBYSCORE")
        .arg(&[TX_SORTED_SET.to_string(), max.to_string(), min.to_string()])
        .query_async::<_, Vec<String>>(&mut conn)
        .await?;

    Ok(value)
}

pub async fn zremrange_by_score(max: u64) -> Result<u64> {
    let mut conn = POOL.get().await?;

    let value: u64 = cmd("ZREMRANGEBYSCORE")
        .arg(&[TX_SORTED_SET.to_string(), "-inf".to_string(), max.to_string()])
        .query_async::<_, u64>(&mut conn)
        .await?;

    Ok(value)
}
