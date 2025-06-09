# HAL9 Performance Optimization Architecture

## Overview

This document outlines the performance optimization strategy for HAL9 to scale from prototype to supporting 1000+ concurrent users with sub-second latency.

## Current Limitations

1. **SQLite Database**: Single-writer limitation, file-based storage
2. **In-Memory Caching**: Limited to single server instance
3. **Synchronous Processing**: Some operations block others
4. **No Connection Pooling**: Database connections created per request

## Target Architecture

```
┌─────────────────────────────────────────────────┐
│                Load Balancer                     │
│              (HAProxy/Nginx)                     │
└─────────────────┬───────────────────────────────┘
                  │
     ┌────────────┼────────────┬─────────────┐
     │            │            │             │
┌────▼────┐ ┌────▼────┐ ┌────▼────┐  ┌─────▼─────┐
│ HAL9    │ │ HAL9    │ │ HAL9    │  │ HAL9      │
│ Server  │ │ Server  │ │ Server  │  │ Server    │
│ Node 1  │ │ Node 2  │ │ Node 3  │  │ Node N    │
└────┬────┘ └────┬────┘ └────┬────┘  └─────┬─────┘
     │           │           │              │
     └───────────┴───────────┴──────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    ┌────▼────┐    ┌────▼────┐    ┌────▼────┐
    │ Redis   │    │ Redis   │    │ Redis   │
    │ Primary │    │ Replica │    │ Replica │
    └─────────┘    └─────────┘    └─────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    ┌────▼────┐    ┌────▼────┐    ┌────▼────┐
    │PostgreSQL│   │PostgreSQL│   │PostgreSQL│
    │ Primary │    │ Replica │    │ Replica │
    └─────────┘    └─────────┘    └─────────┘
```

## Optimization Strategies

### 1. Database Layer Optimization

#### PostgreSQL Migration

**Benefits**:
- Multi-writer support with MVCC
- Advanced indexing (B-tree, Hash, GiST, GIN)
- Parallel query execution
- Connection pooling with PgBouncer
- Streaming replication for read scaling

**Implementation**:
```rust
// Connection pool configuration
pub struct DatabaseConfig {
    pub max_connections: u32,      // 100
    pub min_connections: u32,      // 10
    pub connection_timeout: Duration, // 30s
    pub idle_timeout: Duration,    // 10min
    pub max_lifetime: Duration,    // 30min
}

// PgBouncer configuration
pgbouncer:
  pool_mode: transaction
  max_client_conn: 1000
  default_pool_size: 25
  reserve_pool_size: 5
```

#### Schema Optimization

```sql
-- Optimized tables with proper indexes
CREATE TABLE neurons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    layer VARCHAR(10) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_neurons_layer ON neurons(layer);
CREATE INDEX idx_neurons_created_at ON neurons(created_at DESC);

-- Partitioned tables for time-series data
CREATE TABLE signals (
    id UUID DEFAULT gen_random_uuid(),
    neuron_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    payload JSONB,
    PRIMARY KEY (id, timestamp)
) PARTITION BY RANGE (timestamp);

-- Create monthly partitions
CREATE TABLE signals_2025_01 PARTITION OF signals
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
```

### 2. Caching Layer

#### Redis Implementation

**Cache Strategy**:
```rust
pub enum CacheStrategy {
    // Write-through: Write to cache and DB simultaneously
    WriteThrough,
    
    // Write-behind: Write to cache first, async to DB
    WriteBehind { 
        batch_size: usize,
        flush_interval: Duration,
    },
    
    // Cache-aside: Read from cache, fallback to DB
    CacheAside {
        ttl: Duration,
    },
}

pub struct CacheConfig {
    pub strategy: CacheStrategy,
    pub max_memory: ByteSize,        // 4GB
    pub eviction_policy: EvictionPolicy::LRU,
    pub persistence: RedisPersistence::AOF,
}
```

**Cache Keys Design**:
```
# User data
user:{user_id}                    # User object
user:{user_id}:sessions           # Active sessions
user:{user_id}:api_keys           # API keys

# Neuron data
neuron:{neuron_id}                # Neuron config
neuron:{neuron_id}:state          # Current state
neuron:{neuron_id}:metrics        # Recent metrics

# Signal data
signal:{signal_id}                # Signal object
signal:batch:{batch_id}           # Batch of signals

# Memory system
memory:{neuron_id}:recent         # Recent memories
memory:search:{query_hash}        # Search results cache
```

### 3. Connection Pooling

#### Database Connection Pool
```rust
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connection_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect(&config.database_url)
        .await
}
```

#### Redis Connection Pool
```rust
use redis::aio::ConnectionManager;
use bb8_redis::{bb8, RedisConnectionManager};

pub async fn create_redis_pool(config: &RedisConfig) -> Result<bb8::Pool<RedisConnectionManager>> {
    let manager = RedisConnectionManager::new(config.url.clone())?;
    
    bb8::Pool::builder()
        .max_size(config.max_connections)
        .min_idle(Some(config.min_connections))
        .connection_timeout(config.connection_timeout)
        .build(manager)
        .await
}
```

### 4. Async Processing

#### Message Queue Integration
```rust
// Use Tokio channels for in-process queuing
pub struct AsyncProcessor {
    tx: mpsc::Sender<ProcessingTask>,
    rx: mpsc::Receiver<ProcessingTask>,
    workers: Vec<JoinHandle<()>>,
}

// Use Redis Streams for distributed queuing
pub struct DistributedQueue {
    redis: ConnectionManager,
    stream_key: String,
    consumer_group: String,
}

impl DistributedQueue {
    pub async fn publish(&self, task: &ProcessingTask) -> Result<()> {
        self.redis.xadd(
            &self.stream_key,
            "*",
            &[("data", serde_json::to_string(task)?)]
        ).await?;
        Ok(())
    }
    
    pub async fn consume(&self) -> Result<Vec<ProcessingTask>> {
        // Consumer group processing with acknowledgment
        let results = self.redis.xreadgroup(
            &self.consumer_group,
            &self.consumer_id,
            &[&self.stream_key],
            &[">"],
            None,
            Some(10),
        ).await?;
        
        // Process and acknowledge
        Ok(results)
    }
}
```

### 5. Query Optimization

#### Database Query Patterns
```rust
// Batch operations
pub async fn batch_insert_signals(signals: Vec<Signal>) -> Result<()> {
    let mut tx = pool.begin().await?;
    
    // Use COPY for bulk inserts
    let mut copy = tx.copy_in_raw(
        "COPY signals (id, neuron_id, timestamp, payload) FROM STDIN WITH (FORMAT BINARY)"
    ).await?;
    
    for signal in signals {
        copy.send(signal.to_binary()).await?;
    }
    
    copy.finish().await?;
    tx.commit().await?;
    Ok(())
}

// Prepared statements
pub struct PreparedQueries {
    get_neuron: Statement,
    update_neuron_state: Statement,
    insert_signal: Statement,
}

impl PreparedQueries {
    pub async fn prepare(pool: &PgPool) -> Result<Self> {
        Ok(Self {
            get_neuron: pool.prepare("SELECT * FROM neurons WHERE id = $1").await?,
            update_neuron_state: pool.prepare(
                "UPDATE neurons SET state = $2, updated_at = NOW() WHERE id = $1"
            ).await?,
            insert_signal: pool.prepare(
                "INSERT INTO signals (neuron_id, payload) VALUES ($1, $2)"
            ).await?,
        })
    }
}
```

### 6. Load Balancing

#### HAProxy Configuration
```
global
    maxconn 4096
    nbproc 4
    nbthread 4

defaults
    mode http
    timeout connect 5s
    timeout client 30s
    timeout server 30s
    option httplog

frontend hal9_frontend
    bind *:80
    bind *:443 ssl crt /etc/ssl/hal9.pem
    redirect scheme https if !{ ssl_fc }
    default_backend hal9_backend

backend hal9_backend
    balance leastconn
    option httpchk GET /health
    server hal9_1 10.0.0.1:8080 check
    server hal9_2 10.0.0.2:8080 check
    server hal9_3 10.0.0.3:8080 check
    server hal9_4 10.0.0.4:8080 check
```

### 7. Monitoring & Profiling

#### Performance Metrics
```rust
pub struct PerformanceMetrics {
    // Request metrics
    pub request_duration: Histogram,
    pub request_rate: Counter,
    pub concurrent_requests: Gauge,
    
    // Database metrics
    pub db_query_duration: Histogram,
    pub db_connection_pool_size: Gauge,
    pub db_connection_wait_time: Histogram,
    
    // Cache metrics
    pub cache_hit_rate: Gauge,
    pub cache_miss_count: Counter,
    pub cache_eviction_count: Counter,
    
    // System metrics
    pub cpu_usage: Gauge,
    pub memory_usage: Gauge,
    pub goroutines: Gauge,
}
```

#### Load Testing with k6
```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
    stages: [
        { duration: '30s', target: 100 },  // Ramp up
        { duration: '1m', target: 1000 },  // Stay at 1000 users
        { duration: '30s', target: 0 },    // Ramp down
    ],
    thresholds: {
        http_req_duration: ['p(95)<500'], // 95% of requests under 500ms
        http_req_failed: ['rate<0.01'],   // Error rate under 1%
    },
};

export default function() {
    // Test signal submission
    let signal = {
        content: "Test signal",
        layer: "L4",
        neuron_id: "test-neuron"
    };
    
    let res = http.post('http://localhost:8080/api/v1/signal', 
        JSON.stringify(signal), 
        { headers: { 'Content-Type': 'application/json' } }
    );
    
    check(res, {
        'status is 200': (r) => r.status === 200,
        'response time < 500ms': (r) => r.timings.duration < 500,
    });
    
    sleep(1);
}
```

## Implementation Plan

### Phase 1: Database Migration (2 days)
1. Set up PostgreSQL cluster with replication
2. Migrate schema from SQLite
3. Update connection handling
4. Test data migration scripts

### Phase 2: Caching Layer (2 days)
1. Deploy Redis cluster
2. Implement cache abstraction
3. Add cache warming strategies
4. Monitor cache effectiveness

### Phase 3: Load Testing & Optimization (2 days)
1. Set up k6 test scenarios
2. Identify bottlenecks
3. Optimize critical paths
4. Verify 1000+ user support

## Performance Targets

| Metric | Current | Target | Method |
|--------|---------|--------|--------|
| Response Time (p50) | 200ms | 50ms | Caching, DB optimization |
| Response Time (p99) | 2s | 200ms | Query optimization |
| Throughput | 100 req/s | 10,000 req/s | Horizontal scaling |
| Concurrent Users | 10 | 1000+ | Connection pooling |
| Database Connections | Unlimited | 100 pooled | PgBouncer |
| Cache Hit Rate | 0% | 90%+ | Redis caching |

## Cost Estimation

**Monthly Infrastructure Costs**:
- PostgreSQL (RDS): 3x db.r6g.xlarge = $1,500
- Redis (ElastiCache): 3x cache.r6g.large = $600
- Compute (EC2): 4x c6g.2xlarge = $800
- Load Balancer (ALB): $25
- Monitoring (CloudWatch): $100
- **Total**: ~$3,025/month

## Conclusion

This optimization strategy will transform HAL9 from a prototype into a production-ready system capable of handling enterprise workloads. The combination of PostgreSQL for persistence, Redis for caching, and horizontal scaling enables linear performance improvements as demand grows.