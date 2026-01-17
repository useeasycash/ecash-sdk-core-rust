use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;

/// Cache entry with expiration
struct CacheEntry<T> {
    value: T,
    expiration: Instant,
}

/// In-memory cache for agent quotes and route data
pub struct Cache<T> {
    items: Arc<DashMap<String, CacheEntry<T>>>,
    ttl: Duration,
}

impl<T: Clone + Send + Sync + 'static> Cache<T> {
    /// Creates a new cache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        let cache = Self {
            items: Arc::new(DashMap::new()),
            ttl,
        };

        // Start cleanup task
        let items_clone = cache.items.clone();
        let cleanup_ttl = ttl;
        tokio::spawn(async move {
            let mut interval = time::interval(cleanup_ttl);
            loop {
                interval.tick().await;
                let now = Instant::now();
                items_clone.retain(|_, entry| now < entry.expiration);
            }
        });

        cache
    }

    /// Stores a value in the cache
    pub fn set(&self, key: String, value: T) {
        self.items.insert(
            key,
            CacheEntry {
                value,
                expiration: Instant::now() + self.ttl,
            },
        );
    }

    /// Retrieves a value from the cache
    pub fn get(&self, key: &str) -> Option<T> {
        self.items.get(key).and_then(|entry| {
            if Instant::now() < entry.expiration {
                Some(entry.value.clone())
            } else {
                self.items.remove(key);
                None
            }
        })
    }

    /// Removes a key from the cache
    pub fn delete(&self, key: &str) {
        self.items.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = Cache::new(Duration::from_millis(100));
        cache.set("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get("key1"), None);
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set("key1".to_string(), "value1".to_string());
        cache.delete("key1");
        assert_eq!(cache.get("key1"), None);
    }

    #[tokio::test]
    async fn test_cache_multiple_keys() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), Some("value2".to_string()));
    }
}
