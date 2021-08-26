use deadpool_redis::redis::{cmd, RedisError};
use deadpool_redis::{Config, Pool};
use once_cell::sync::Lazy;
use std::sync::Arc;

const REDIS_URL: &str = "redis://0.0.0.0:6901";

static POOL: Lazy<Arc<Pool>> = Lazy::new(|| {
    let cfg = Config::from_url(REDIS_URL);
    let pool = cfg.create_pool().unwrap();

    Arc::new(pool)
});

pub async fn test(_key: &str) -> Result<(), RedisError> {
    let mut conn = POOL.get().await.unwrap();

    cmd("SET")
        .arg(&["deadpool/test_key", "42"])
        .query_async::<_, ()>(&mut conn)
        .await
        .unwrap();

    let value: String = cmd("GET")
        .arg(&["deadpool/test_key"])
        .query_async(&mut conn)
        .await
        .unwrap();

    println!("Value: {}", value);

    Ok(())
}
