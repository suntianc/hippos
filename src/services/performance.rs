//! Performance Optimization Module
//!
//! Provides caching, batch operations, and performance monitoring for the memory system.
//! Implements multi-level caching (L1 in-memory, L2 distributed) and batch processing
//! to optimize performance for high-throughput scenarios.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable L1 in-memory cache
    pub cache_enabled: bool,
    /// L1 cache maximum size (number of entries)
    pub cache_max_size: usize,
    /// L1 cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable batch operations
    pub batch_enabled: bool,
    /// Batch size threshold
    pub batch_size_threshold: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
    /// Enable query result caching
    pub query_cache_enabled: bool,
    /// Query cache TTL in seconds
    pub query_cache_ttl_seconds: u64,
    /// Connection pool size
    pub pool_size: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_max_size: 10000,
            cache_ttl_seconds: 300,
            batch_enabled: true,
            batch_size_threshold: 100,
            batch_timeout_ms: 50,
            query_cache_enabled: true,
            query_cache_ttl_seconds: 60,
            pool_size: 10,
        }
    }
}

/// Performance statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_evictions: u64,
    pub batch_operations: u64,
    pub batch_items_processed: u64,
    pub avg_query_time_ms: f64,
    pub avg_batch_time_ms: f64,
    pub total_queries: u64,
    pub active_connections: u32,
    pub memory_usage_bytes: u64,
}

/// Performance snapshot for monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: String,
    pub stats: PerformanceStats,
    pub cache_hit_rate: f64,
    pub queries_per_second: f64,
    pub active_connections: u32,
}

/// Cache entry with TTL
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// L1 In-memory cache with TTL support
#[derive(Debug)]
pub struct MemoryCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    config: PerformanceConfig,
    cache: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    stats: Arc<RwLock<PerformanceStats>>,
}

impl<K, V> MemoryCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new memory cache
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config: config.clone(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PerformanceStats::default())),
        }
    }

    /// Get a value from cache
    pub async fn get(&self, key: &K) -> Option<V> {
        if !self.config.cache_enabled {
            return None;
        }

        let mut cache = self.cache.write().await;
        if let Some(entry) = cache.get(key) {
            if entry.is_expired() {
                cache.remove(key);
                {
                    let mut stats = self.stats.write().await;
                    stats.cache_evictions += 1;
                }
                return None;
            }
            {
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
            }
            return Some(entry.value.clone());
        }

        {
            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
        }
        None
    }

    /// Set a value in cache
    pub async fn set(&self, key: K, value: V) {
        if !self.config.cache_enabled {
            return;
        }

        let mut cache = self.cache.write().await;

        if cache.len() >= self.config.cache_max_size {
            self.evict_expired(&mut cache).await;
            if cache.len() >= self.config.cache_max_size {
                let to_remove = cache.len() / 10;
                let keys: Vec<_> = cache
                    .keys()
                    .take(to_remove)
                    .cloned()
                    .collect();
                for key in keys {
                    cache.remove(&key);
                }
                {
                    let mut stats = self.stats.write().await;
                    stats.cache_evictions += to_remove as u64;
                }
            }
        }

        let entry = CacheEntry::new(value, Duration::from_secs(self.config.cache_ttl_seconds));
        cache.insert(key, entry);
    }

    /// Remove a value from cache
    pub async fn remove(&self, key: &K) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache size
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        let stats_guard = self.stats.read().await;
        let stats = (*stats_guard).clone();
        drop(stats_guard);
        let cache_size = self.cache.read().await.len();
        let mut stats = stats;
        stats.memory_usage_bytes = cache_size as u64 * std::mem::size_of::<(K, CacheEntry<V>)>() as u64;
        stats
    }

    /// Evict expired entries
    async fn evict_expired(&self, cache: &mut HashMap<K, CacheEntry<V>>) {
        let expired_keys: Vec<K> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        for key in &expired_keys {
            cache.remove(key);
        }

        if !expired_keys.is_empty() {
            let mut stats = self.stats.write().await;
            stats.cache_evictions += expired_keys.len() as u64;
        }
    }

    /// Cleanup task - runs periodically to remove expired entries
    pub async fn cleanup_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let mut cache = self.cache.write().await;
            self.evict_expired(&mut cache).await;
            debug!(
                "Cache cleanup: {} entries, {} evictions",
                cache.len(),
                self.stats.read().await.cache_evictions
            );
        }
    }
}

/// Batch operation result
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchResult<T> {
    pub success_count: usize,
    pub failure_count: usize,
    pub results: Vec<Result<T, String>>,
    pub duration_ms: u64,
}

/// Batch operation handler
#[derive(Debug)]
pub struct BatchProcessor<K, V, F, Fut>
where
    K: Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
    F: Fn(K) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<V, String>> + Send,
{
    config: PerformanceConfig,
    pending: Arc<RwLock<HashMap<K, (Instant, Fut)>>>,
    processor: Arc<F>,
    results: Arc<RwLock<HashMap<K, Result<V, String>>>>,
    stats: Arc<RwLock<PerformanceStats>>,
}

impl<K, V, F, Fut> BatchProcessor<K, V, F, Fut>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(K) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<V, String>> + Send,
{
    /// Create a new batch processor
    pub fn new(config: PerformanceConfig, processor: F) -> Self {
        Self {
            config,
            pending: Arc::new(RwLock::new(HashMap::new())),
            processor: Arc::new(processor),
            results: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PerformanceStats::default())),
        }
    }

    /// Queue an item for batch processing
    pub async fn queue(&self, key: K, item: V) {
        if !self.config.batch_enabled {
            // Process immediately if batch is disabled
            let result = (self.processor)(key.clone()).await;
            let mut results = self.results.write().await;
            results.insert(key, result);
            return;
        }

        let mut pending = self.pending.write().await;
        pending.insert(key.clone(), (Instant::now(), (self.processor)(key)));
    }

    /// Queue multiple items for batch processing
    pub async fn queue_batch(&self, items: Vec<(K, V)>) {
        if !self.config.batch_enabled {
            // Process immediately if batch is disabled
            for (key, _) in items {
                let result = (self.processor)(key.clone()).await;
                let mut results = self.results.write().await;
                results.insert(key, result);
            }
            return;
        }

        let mut pending = self.pending.write().await;
        for (key, _) in items {
            if !pending.contains_key(&key) {
                pending.insert(key.clone(), (Instant::now(), (self.processor)(key)));
            }
        }
    }

    /// Process pending items
    pub async fn process_pending(&self) -> BatchResult<V> {
        let start = Instant::now();
        let mut pending = self.pending.write().await;
        let mut results = self.results.write().await;

        // Store pending length before drain to avoid borrow conflict
        let current_pending_len = pending.len();

        // Collect items to process
        let items: Vec<(K, Instant, Fut)> = pending
            .drain()
            .filter(|(_, (created_at, _))| {
                created_at.elapsed() > Duration::from_millis(self.config.batch_timeout_ms)
                    || current_pending_len >= self.config.batch_size_threshold
            })
            .map(|(k, v)| (k, v.0, v.1))
            .collect();

        drop(pending);

        let mut batch_results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        for (key, _, future) in items {
            match future.await {
                Ok(value) => {
                    results.insert(key.clone(), Ok(value.clone()));
                    batch_results.push(Ok(value));
                    success_count += 1;
                }
                Err(e) => {
                    results.insert(key.clone(), Err(e.clone()));
                    batch_results.push(Err(e));
                    failure_count += 1;
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        {
            let mut stats = self.stats.write().await;
            stats.batch_operations += 1;
            stats.batch_items_processed += success_count as u64;
            stats.avg_batch_time_ms =
                (stats.avg_batch_time_ms * (stats.batch_operations - 1) as f64 + duration_ms as f64)
                    / stats.batch_operations as f64;
        }

        BatchResult {
            success_count,
            failure_count,
            results: batch_results,
            duration_ms,
        }
    }

    /// Get result for a specific key
    pub async fn get_result(&self, key: &K) -> Option<Result<V, String>> {
        let results = self.results.read().await;
        results.get(key).cloned()
    }

    /// Get batch processor statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        let stats_guard = self.stats.read().await;
        let stats = (*stats_guard).clone();
        drop(stats_guard);
        stats
    }

    /// Get pending count
    pub async fn pending_count(&self) -> usize {
        self.pending.read().await.len()
    }
}

/// Query result cache for expensive operations
#[derive(Debug)]
pub struct QueryCache<Q, R>
where
    Q: Hash + Eq + Clone + Send + Sync,
    R: Clone + Send + Sync,
{
    config: PerformanceConfig,
    cache: Arc<MemoryCache<Q, R>>,
}

impl<Q, R> QueryCache<Q, R>
where
    Q: Hash + Eq + Clone + Send + Sync + 'static,
    R: Clone + Send + Sync + 'static,
{
    /// Create a new query cache
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config: config.clone(),
            cache: Arc::new(MemoryCache::new(config.clone())),
        }
    }

    /// Get cached query result
    pub async fn get(&self, query: &Q) -> Option<R> {
        if !self.config.query_cache_enabled {
            return None;
        }
        self.cache.get(query).await
    }

    /// Cache query result
    pub async fn set(&self, query: Q, result: R) {
        if !self.config.query_cache_enabled {
            return;
        }
        self.cache.set(query, result).await
    }

    /// Invalidate cached result
    pub async fn invalidate(&self, query: &Q) {
        self.cache.remove(query).await
    }

    /// Clear all cached results
    pub async fn clear(&self) {
        self.cache.clear().await
    }

    /// Get query cache statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        self.cache.get_stats().await
    }
}

/// Connection pool for database connections
#[derive(Debug)]
pub struct ConnectionPool<T: Clone> {
    config: PerformanceConfig,
    pool: Arc<RwLock<Vec<T>>>,
    in_use: Arc<RwLock<HashSet<usize>>>,
    stats: Arc<RwLock<PerformanceStats>>,
    factory: T,
}

impl<T: Clone + Send> ConnectionPool<T> {
    /// Create a new connection pool
    pub fn new(config: PerformanceConfig, factory: T) -> Self {
        Self {
            config: config.clone(),
            pool: Arc::new(RwLock::new(Vec::new())),
            in_use: Arc::new(RwLock::new(HashSet::new())),
            stats: Arc::new(RwLock::new(PerformanceStats::default())),
            factory,
        }
    }

    /// Get a connection from the pool
    pub async fn get(&self) -> Option<PooledConnection<T>> {
        let mut pool = self.pool.write().await;
        let mut in_use = self.in_use.write().await;

        // Find available connection
        for (i, _) in pool.iter().enumerate() {
            if !in_use.contains(&i) {
                in_use.insert(i);
                let mut stats = self.stats.write().await;
                stats.active_connections = in_use.len() as u32;
                return Some(PooledConnection {
                    index: i,
                    pool: self.pool.clone(),
                    in_use: self.in_use.clone(),
                    stats: self.stats.clone(),
                });
            }
        }

        // Create new connection if pool is not full
        if (pool.len() as u32) < self.config.pool_size {
            let index = pool.len();
            pool.push(self.factory.clone());
            in_use.insert(index);
            let mut stats = self.stats.write().await;
            stats.active_connections = in_use.len() as u32;
            return Some(PooledConnection {
                index,
                pool: self.pool.clone(),
                in_use: self.in_use.clone(),
                stats: self.stats.clone(),
            });
        }

        None
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        let stats_guard = self.stats.read().await;
        let stats = (*stats_guard).clone();
        drop(stats_guard);
        let pool = self.pool.read().await;
        let mut stats = stats;
        let in_use_guard = self.in_use.read().await;
        stats.active_connections = in_use_guard.len() as u32;
        drop(in_use_guard);
        stats
    }
}

/// Guard for pooled connection - releases connection when dropped
pub struct PooledConnection<T: Clone + Send> {
    index: usize,
    pool: Arc<RwLock<Vec<T>>>,
    in_use: Arc<RwLock<HashSet<usize>>>,
    stats: Arc<RwLock<PerformanceStats>>,
}

impl<T: Clone + Send> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Ok(mut in_use) = self.in_use.try_write() {
            in_use.remove(&self.index);
            if let Ok(mut stats) = self.stats.try_write() {
                stats.active_connections = in_use.len() as u32;
            }
        }
    }
}

impl<T: Clone + Send> PooledConnection<T> {
    /// Get the inner connection
    pub fn get(&self) -> T {
        todo!("Implement based on actual connection type")
    }
}

/// Performance monitoring service
#[derive(Debug)]
pub struct PerformanceMonitor {
    config: PerformanceConfig,
    start_time: Instant,
    query_times: Arc<RwLock<Vec<Duration>>>,
    stats: Arc<RwLock<PerformanceStats>>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            query_times: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(PerformanceStats::default())),
        }
    }

    /// Record a query execution time
    pub async fn record_query(&self, duration: Duration) {
        let mut query_times = self.query_times.write().await;
        query_times.push(duration);

        // Keep only last 1000 query times
        if query_times.len() > 1000 {
            let current_len = query_times.len();
            query_times.drain(0..(current_len - 1000));
        }

        let mut stats = self.stats.write().await;
        stats.total_queries += 1;
        stats.avg_query_time_ms =
            (stats.avg_query_time_ms * (stats.total_queries - 1) as f64 + duration.as_secs_f64() * 1000.0)
                / stats.total_queries as f64;
    }

    /// Get current performance snapshot
    pub async fn snapshot(&self) -> PerformanceSnapshot {
        let stats_guard = self.stats.read().await;
        let stats = (*stats_guard).clone();
        drop(stats_guard);
        let query_times = self.query_times.read().await;

        let cache_hit_rate = if stats.cache_hits + stats.cache_misses > 0 {
            stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64
        } else {
            0.0
        };

        let uptime_secs = self.start_time.elapsed().as_secs_f64();
        let queries_per_second = if uptime_secs > 0.0 {
            stats.total_queries as f64 / uptime_secs
        } else {
            0.0
        };

        let active_connections = stats.active_connections;

        PerformanceSnapshot {
            timestamp: chrono::Utc::now().to_rfc3339(),
            stats,
            cache_hit_rate,
            queries_per_second,
            active_connections,
        }
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        let stats_guard = self.stats.read().await;
        let stats = (*stats_guard).clone();
        drop(stats_guard);
        stats
    }

    /// Get recent query times (for percentile calculation)
    pub async fn get_recent_query_times(&self) -> Vec<Duration> {
        let times_guard = self.query_times.read().await;
        let times = (*times_guard).clone();
        drop(times_guard);
        times
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            start_time: self.start_time,
            query_times: self.query_times.clone(),
            stats: self.stats.clone(),
        }
    }
}
