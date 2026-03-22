use deadpool_redis::{Runtime, redis::AsyncCommands};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::api::{error, messages};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub access_token_expiration: u64,
    pub refresh_token_expiration: u64,
    pub cookie_secure: bool,
    pub database_url: String,
    pub redis_url: String,
    pub frontend_url: String,
    pub ip: String,
    pub port: u16,
    pub cloudinary_url: Option<String>,
    pub migration_path: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, error::SystemError> {
        let settings = config::Config::builder()
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .map_err(|e| {
                error::SystemError::internal_error(format!("Không thể tải cấu hình: {e}"))
            })?;

        let cookie_secure = compute_cookie_secure(
            settings.get_string("COOKIE_SECURE").ok().as_deref(),
            settings.get_string("APP_ENV").ok().as_deref(),
        );

        let jwt_secret = settings.get_string("SECRET_KEY").map_err(|_| {
            error::SystemError::internal_error_key(messages::i18n::Key::ConfigSecretKeyMissing)
        })?;

        let database_url = settings.get_string("DATABASE_URL").map_err(|_| {
            error::SystemError::internal_error_key(messages::i18n::Key::ConfigDatabaseUrlMissing)
        })?;

        let redis_url = settings.get_string("REDIS_URL").map_err(|_| {
            error::SystemError::internal_error_key(messages::i18n::Key::ConfigRedisUrlMissing)
        })?;

        let config = Self {
            jwt_secret,
            access_token_expiration: settings
                .get::<u64>("ACCESS_TOKEN_EXPIRATION")
                .unwrap_or(900),
            refresh_token_expiration: settings
                .get::<u64>("REFRESH_TOKEN_EXPIRATION")
                .unwrap_or(604800),
            cookie_secure,
            database_url,
            redis_url,
            frontend_url: settings
                .get_string("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            ip: settings
                .get_string("IP")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: settings.get::<u16>("PORT").unwrap_or(8080),
            cloudinary_url: settings.get_string("CLOUDINARY_URL").ok(),
            migration_path: settings
                .get_string("MIGRATION_PATH")
                .unwrap_or_else(|_| "./migrations".to_string()),
        };

        config.validate()?;
        Ok(config)
    }

    pub fn from_env_lossy() -> Self {
        Self::from_env().unwrap_or_else(|_| Self {
            jwt_secret: "dev-secret-key".to_string(),
            access_token_expiration: 900,
            refresh_token_expiration: 604800,
            cookie_secure: false,
            database_url: "postgres://localhost/appchat".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            frontend_url: "http://localhost:5173".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 8080,
            cloudinary_url: None,
            migration_path: "./migrations".to_string(),
        })
    }

    fn validate(&self) -> Result<(), error::SystemError> {
        if self.jwt_secret.trim().is_empty() {
            return Err(error::SystemError::internal_error_key(
                messages::i18n::Key::ConfigSecretKeyEmpty,
            ));
        }

        if self.access_token_expiration == 0 || self.refresh_token_expiration == 0 {
            return Err(error::SystemError::internal_error_key(
                messages::i18n::Key::ConfigTokenExpirationInvalid,
            ));
        }

        if self.database_url.trim().is_empty() || self.redis_url.trim().is_empty() {
            return Err(error::SystemError::internal_error_key(
                messages::i18n::Key::ConfigDatabaseRedisEmpty,
            ));
        }

        Ok(())
    }
}

fn compute_cookie_secure(cookie_secure: Option<&str>, app_env: Option<&str>) -> bool {
    if let Some(value) = cookie_secure {
        let normalized = value.to_ascii_lowercase();
        return normalized == "1" || normalized == "true" || normalized == "yes";
    }

    app_env
        .map(|env| env.eq_ignore_ascii_case("production"))
        .unwrap_or(false)
}

#[allow(dead_code)]
pub trait DbPool {
    fn as_pool(&self) -> &PgPool;
}

impl DbPool for PgPool {
    fn as_pool(&self) -> &PgPool {
        self
    }
}

#[async_trait::async_trait]
pub trait CacheStore {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, error::SystemError>
    where
        T: serde::de::DeserializeOwned + Send;

    async fn set<T>(
        &self,
        key: &str,
        value: &T,
        expiration: usize,
    ) -> Result<(), error::SystemError>
    where
        T: serde::Serialize + Send + Sync;

    async fn delete(&self, key: &str) -> Result<(), error::SystemError>;
}

#[allow(dead_code)]
pub async fn connect_database() -> Result<PgPool, error::SystemError> {
    let config = AppConfig::from_env()?;
    connect_database_with_config(&config).await
}

pub async fn connect_database_with_config(
    config: &AppConfig,
) -> Result<PgPool, error::SystemError> {
    let database_url = &config.database_url;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(2)
        .acquire_slow_threshold(std::time::Duration::from_secs(3))
        .connect(database_url)
        .await?;

    tracing::info!("Đang chạy Database Migrations...");
    let migrator = sqlx::migrate::Migrator::new(std::path::Path::new(&config.migration_path))
        .await
        .map_err(|e| {
            tracing::error!("Lỗi đọc migration path '{}': {}", config.migration_path, e);
            error::SystemError::internal_error_key(messages::i18n::Key::MigrationFilesLoadFailed)
        })?;

    migrator.run(&pool).await.map_err(|e| {
        tracing::error!("Lỗi chạy migrations: {}", e);
        error::SystemError::internal_error_key(messages::i18n::Key::DatabaseSchemaInitFailed)
    })?;

    Ok(pool)
}

#[derive(Clone)]
pub struct RedisCache {
    pool: deadpool_redis::Pool,
}

impl RedisCache {
    pub async fn new() -> Result<Self, error::SystemError> {
        let config = AppConfig::from_env()?;
        Self::new_with_config(&config).await
    }

    pub async fn new_with_config(config: &AppConfig) -> Result<Self, error::SystemError> {
        let mut cfg = deadpool_redis::Config::from_url(&config.redis_url);
        cfg.pool = Some(deadpool_redis::PoolConfig {
            max_size: 16,
            ..Default::default()
        });
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(Self { pool })
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, error::SystemError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut conn = self.pool.get().await?;

        let value: Option<Vec<u8>> = conn.get(key).await?;

        match value {
            Some(v) => {
                let parsed = serde_json::from_slice(&v)?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T>(
        &self,
        key: &str,
        value: &T,
        expiration: usize,
    ) -> Result<(), error::SystemError>
    where
        T: serde::Serialize,
    {
        let mut conn = self.pool.get().await?;

        let serialized = serde_json::to_vec(value)?;

        conn.set_ex::<_, _, ()>(key, serialized, expiration as u64)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), error::SystemError> {
        let mut conn = self.pool.get().await?;
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    /// Expose Redis pool cho PresenceService
    pub fn get_pool(&self) -> &deadpool_redis::Pool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl CacheStore for RedisCache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, error::SystemError>
    where
        T: serde::de::DeserializeOwned + Send,
    {
        RedisCache::get(self, key).await
    }

    async fn set<T>(
        &self,
        key: &str,
        value: &T,
        expiration: usize,
    ) -> Result<(), error::SystemError>
    where
        T: serde::Serialize + Send + Sync,
    {
        RedisCache::set(self, key, value, expiration).await
    }

    async fn delete(&self, key: &str) -> Result<(), error::SystemError> {
        RedisCache::delete(self, key).await
    }
}
