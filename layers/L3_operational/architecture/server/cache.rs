//! Compatibility layer for cache module
//! Re-exports simple_cache to maintain compatibility

pub use crate::simple_cache::{
    CachePool,
    CacheConfig,
    CacheKeys,
    RedisPool,
    PoolMetrics,
};