# External Dependencies Analysis - 2HAL9 Project
Date: 2025-06-17

## Executive Summary

The 2HAL9 project has several external dependencies, but most already have mock implementations in place. The project is designed with a hybrid approach that allows switching between real services and mocks through configuration.

## 1. Claude API (Anthropic)

### Usage Locations:
- `/layers/L3_operational/architecture/server/claude.rs` - Main Claude integration
- Used across 39 files for AI reasoning

### Current Implementation:
```rust
// Line 271: External API call
.post("https://api.anthropic.com/v1/messages")
```

### Existing Mock Implementation:
✅ **MockClaude** already exists (lines 33-137)
- Layer-specific mock responses
- Configurable delays
- Custom response mapping

### Replacement Strategy:
```rust
// Already implemented in HybridClaude (lines 321-441)
pub enum ClaudeMode {
    Mock,      // Always use mock
    Api,       // Always use API
    Auto,      // Use API in production, mock in dev
    Hybrid,    // Use API with automatic fallback to mock
}
```

**To use mock only:** Set environment variable `HAL9_CLAUDE_MODE=mock` or configure in TOML.

## 2. Ollama Integration

### Usage Locations:
- `/test_ollama_integration.rs` - Test file for Ollama provider
- `/competitions/test_ollama_game.rs` - Game testing with Ollama

### Current Implementation:
```rust
// Line 11: Local Ollama connection
OllamaProvider::new("http://localhost:11434", "llama2")
```

### Replacement Strategy:
- Ollama is **already local** - runs on `localhost:11434`
- No external dependency, just requires local Ollama installation
- Can create a mock provider if needed for testing without Ollama

## 3. PostgreSQL Database

### Usage Locations:
- `/layers/L3_operational/architecture/server/database.rs` - Database abstraction
- Used in 53 files for persistence

### Current Implementation:
```rust
// Supports both SQLite and PostgreSQL
pub enum DatabasePool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
}
```

### Existing Local Alternative:
✅ **SQLite support already implemented**
- Default configuration uses SQLite: `"sqlite:data/hal9.db?mode=rwc"`
- No external database required

**To use SQLite:** Default behavior, no changes needed.

## 4. Redis Cache

### Usage Locations:
- `/layers/L3_operational/architecture/server/cache.rs` - Cache abstraction
- Used in 17 files for caching

### Current Implementation:
```rust
// Line 39: Redis connection
url: "redis://127.0.0.1:6379".to_string()
```

### Replacement Options:

1. **In-Memory Cache** (Quick implementation):
```rust
pub struct InMemoryCache {
    data: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    ttl: Duration,
}
```

2. **SQLite-based Cache** (Persistent):
```rust
// Use SQLite with expiration timestamps
CREATE TABLE cache (
    key TEXT PRIMARY KEY,
    value TEXT,
    expires_at INTEGER
);
```

3. **No Cache Mode**: Most operations work without cache, just slower

## 5. HTTP Clients (reqwest)

### Usage Locations:
- Used in 16 files for external HTTP calls
- Mainly for Claude API and health checks

### Already Handled:
- Claude API calls can use MockClaude
- Health checks can be disabled in local mode
- Most HTTP usage is for optional features

## Minimal Working Demo Configuration

Create a `local-demo.toml` configuration:

```toml
[claude]
mode = "mock"

[database]
type = "sqlite"
url = "sqlite:data/demo.db?mode=rwc"

[cache]
enabled = false  # Or implement InMemoryCache

[server]
port = 8080
host = "127.0.0.1"
```

## Implementation Priority

1. **No changes needed for:**
   - Claude API (use existing MockClaude)
   - PostgreSQL (use existing SQLite support)
   - Ollama (already local)

2. **Minor implementation needed:**
   - Redis replacement with InMemoryCache (~50 lines of code)
   - Or simply disable caching for demo

3. **Configuration:**
   - Create `local-demo.toml` with mock settings
   - Set `HAL9_ENV=development` to trigger mock mode

## Quick Start Commands

```bash
# Set environment for local demo
export HAL9_ENV=development
export HAL9_CLAUDE_MODE=mock

# Use SQLite instead of PostgreSQL (default)
# No additional config needed

# Run without Redis (disable cache)
export HAL9_CACHE_ENABLED=false

# Or implement simple in-memory cache
# (see implementation suggestion above)
```

## Conclusion

The 2HAL9 project is already well-architected for local/offline operation:
- ✅ Claude API has full mock implementation
- ✅ Database supports SQLite (no external DB needed)
- ✅ Ollama is already local
- ⚠️ Redis cache needs simple in-memory replacement (optional)

Total effort needed: **Minimal** - mostly configuration changes and potentially ~50 lines for in-memory cache.