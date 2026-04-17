use async_trait::async_trait;
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, LazyLock};

#[allow(dead_code)]
pub static CACHE: LazyLock<Arc<dyn CacheBackend>> = LazyLock::new(|| {
    #[cfg(feature = "redis")]
    if crate::CONFIG.redis_enabled() {
        if let Ok(redis_cache) = RedisCache::new(&crate::CONFIG.redis_url()) {
            return Arc::new(redis_cache);
        }
    }
    Arc::new(InMemoryCache::new())
});

#[async_trait]
#[allow(dead_code)]
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: &str, expiry_secs: u64) -> Result<(), String>;
    async fn del(&self, key: &str) -> Result<(), String>;
    async fn increment(&self, key: &str, amount: u64, expiry_secs: u64) -> Result<u64, String>;
    async fn publish(&self, channel: &str, payload: &str) -> Result<(), String>;
}

#[allow(dead_code)]
pub struct InMemoryCache {
    store: DashMap<String, (String, u64)>, // (value, timestamp_expiry_secs)
}

impl InMemoryCache {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            store: DashMap::new(),
        }
    }
}

#[async_trait]
impl CacheBackend for InMemoryCache {
    async fn get(&self, key: &str) -> Option<String> {
        if let Some(entry) = self.store.get(key) {
            let (val, exp) = entry.value();
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if exp == &0 || now < *exp {
                return Some(val.clone());
            } else {
                drop(entry);
                self.store.remove(key); // Evict expired
            }
        }
        None
    }

    async fn set(&self, key: &str, value: &str, expiry_secs: u64) -> Result<(), String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let exp = if expiry_secs > 0 { now + expiry_secs } else { 0 };
        self.store.insert(key.to_string(), (value.to_string(), exp));
        Ok(())
    }

    async fn del(&self, key: &str) -> Result<(), String> {
        self.store.remove(key);
        Ok(())
    }

    async fn increment(&self, key: &str, amount: u64, expiry_secs: u64) -> Result<u64, String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let exp = if expiry_secs > 0 { now + expiry_secs } else { 0 };
        
        let mut new_val = amount;
        if let Some(mut entry) = self.store.get_mut(key) {
            if entry.1 == 0 || now < entry.1 {
                if let Ok(current) = entry.0.parse::<u64>() {
                    new_val = current + amount;
                }
            }
            entry.0 = new_val.to_string();
            entry.1 = exp; // Extend expiry (or reset it)
        } else {
            self.store.insert(key.to_string(), (new_val.to_string(), exp));
        }
        Ok(new_val)
    }

    async fn publish(&self, _channel: &str, _payload: &str) -> Result<(), String> {
        // InMemoryCache does not support cross-pod publish.
        // We handle local-only fallback directly in the caller (notifications.rs).
        Ok(())
    }
}

#[cfg(feature = "redis")]
#[allow(dead_code)]
pub struct RedisCache {
    pool: deadpool_redis::Pool,
}

#[cfg(feature = "redis")]
impl RedisCache {
    #[allow(dead_code)]
    pub fn new(url: &str) -> Result<Self, String> {
        let cfg = deadpool_redis::Config::from_url(url);
        let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1)).map_err(|e| e.to_string())?;
        Ok(Self { pool })
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl CacheBackend for RedisCache {
    async fn get(&self, key: &str) -> Option<String> {
        if let Ok(mut conn) = self.pool.get().await {
            let res: redis::RedisResult<Option<String>> = redis::cmd("GET").arg(key).query_async(&mut conn).await;
            return res.unwrap_or(None);
        }
        None
    }

    async fn set(&self, key: &str, value: &str, expiry_secs: u64) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        if expiry_secs > 0 {
            let _: () = redis::cmd("SETEX").arg(key).arg(expiry_secs).arg(value).query_async(&mut conn).await.map_err(|e| e.to_string())?;
        } else {
            let _: () = redis::cmd("SET").arg(key).arg(value).query_async(&mut conn).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn del(&self, key: &str) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let _: () = redis::cmd("DEL").arg(key).query_async(&mut conn).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn increment(&self, key: &str, amount: u64, expiry_secs: u64) -> Result<u64, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let res: u64 = redis::cmd("INCRBY").arg(key).arg(amount).query_async(&mut conn).await.map_err(|e| e.to_string())?;
        if res == amount && expiry_secs > 0 {
            let _: () = redis::cmd("EXPIRE").arg(key).arg(expiry_secs).query_async(&mut conn).await.map_err(|e| e.to_string())?;
        }
        Ok(res)
    }

    async fn publish(&self, channel: &str, payload: &str) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let _: () = redis::cmd("PUBLISH").arg(channel).arg(payload).query_async(&mut conn).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
