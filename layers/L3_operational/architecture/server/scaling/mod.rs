//! Distributed scaling components for 1000+ users

pub mod sharding;
pub mod load_balancer;
pub mod session_manager;
pub mod connection_pool;
pub mod geo_routing;
pub mod health_check;

pub use sharding::{ShardingStrategy, ShardConfig, ShardingManager};
pub use load_balancer::{LoadBalancer, LoadBalancingStrategy};
pub use session_manager::{DistributedSessionManager, Session};
pub use connection_pool::{OptimizedConnectionPool, PoolConfig};
pub use geo_routing::{GeoRouter, Region};
pub use health_check::{HealthChecker, HealthStatus};

#[cfg(test)]
mod tests;