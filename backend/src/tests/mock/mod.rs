pub mod database;

use crate::configs::{AppConfig, RedisCache};

pub fn lazy_mock_pool() -> sqlx::PgPool {
	database::MockDatabase::new().pool()
}

pub async fn test_redis_cache() -> RedisCache {
	RedisCache::new_with_config(&AppConfig::from_env_lossy())
		.await
		.expect("failed to initialize redis cache pool")
}
