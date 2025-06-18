# HAL9 Database Schema Documentation

## 📋 Overview

HAL9 uses a relational database to store system state, user data, game information, and neuron signals. The system supports both PostgreSQL (recommended for production) and SQLite (for development).

## 🗄️ Database Support

| Database | Use Case | Features |
|----------|----------|----------|
| PostgreSQL | Production | Partitioning, Full-text search, JSON support, High concurrency |
| SQLite | Development | Lightweight, No setup required, Good for testing |

## 📊 Core Tables

### 1. System Tables

#### `users`
Stores user account information.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| username | VARCHAR(255) | Unique username |
| email | VARCHAR(255) | Unique email address |
| password_hash | VARCHAR(255) | Bcrypt hashed password |
| role | VARCHAR(50) | User role (admin, user, etc.) |
| is_active | BOOLEAN | Account active status |
| email_verified | BOOLEAN | Email verification status |
| created_at | TIMESTAMP | Account creation time |
| updated_at | TIMESTAMP | Last update time |
| last_login | TIMESTAMP | Last login time |

#### `sessions`
Active user sessions for authentication.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| user_id | UUID/TEXT | References users.id |
| token_hash | VARCHAR(255) | Hashed session token |
| ip_address | INET/TEXT | Client IP address |
| user_agent | TEXT | Client user agent |
| expires_at | TIMESTAMP | Session expiration |
| created_at | TIMESTAMP | Session creation time |

#### `api_keys`
API keys for programmatic access.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| user_id | UUID/TEXT | References users.id |
| name | VARCHAR(255) | Key description |
| key_hash | VARCHAR(255) | Hashed API key |
| permissions | JSON | Permission array |
| is_active | BOOLEAN | Key active status |
| expires_at | TIMESTAMP | Key expiration |

### 2. Neuron System Tables

#### `neurons`
Neuron instances in the HAL9 system.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| layer | VARCHAR(10) | Layer identifier (L1-L9) |
| system_prompt | TEXT | Neuron's system prompt |
| settings | JSON | Configuration settings |
| state | VARCHAR(50) | Current state |
| created_at | TIMESTAMP | Creation time |
| updated_at | TIMESTAMP | Last update time |

#### `signals`
Inter-neuron communication signals (partitioned in PostgreSQL).

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| from_neuron | VARCHAR(255) | Source neuron ID |
| to_neuron | VARCHAR(255) | Target neuron ID |
| layer_from | VARCHAR(10) | Source layer |
| layer_to | VARCHAR(10) | Target layer |
| propagation_type | VARCHAR(20) | Signal type |
| content | TEXT | Signal content |
| metadata | JSON | Additional metadata |
| timestamp | TIMESTAMP | Signal time |

#### `memories`
Long-term memory storage for neurons.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| neuron_id | VARCHAR(255) | Owning neuron |
| content | TEXT | Memory content |
| importance | FLOAT | Importance score (0-1) |
| context | JSON | Contextual information |
| embedding | VECTOR(1536) | Vector embedding (PostgreSQL) |
| created_at | TIMESTAMP | Creation time |

### 3. AI Genius Game Tables

#### `games`
Game instances.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| host_id | UUID/TEXT | Game creator |
| name | VARCHAR(255) | Game name |
| status | VARCHAR(50) | waiting/in_progress/completed |
| max_players | INTEGER | Maximum players (2-12) |
| current_round | INTEGER | Current round number |
| total_rounds | INTEGER | Total rounds (5-50) |
| time_limit_seconds | INTEGER | Time per round |
| settings | JSON | Game configuration |
| created_at | TIMESTAMP | Creation time |
| started_at | TIMESTAMP | Start time |
| ended_at | TIMESTAMP | End time |

#### `game_players`
Players in each game.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| game_id | UUID/TEXT | References games.id |
| user_id | UUID/TEXT | References users.id |
| player_name | VARCHAR(255) | Display name |
| avatar_url | TEXT | Avatar image URL |
| score | INTEGER | Current score |
| is_ready | BOOLEAN | Ready status |
| is_active | BOOLEAN | Active in game |
| joined_at | TIMESTAMP | Join time |

#### `game_rounds`
Individual game rounds.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| game_id | UUID/TEXT | References games.id |
| round_number | INTEGER | Round number |
| prompt | TEXT | Round prompt |
| category | VARCHAR(100) | Prompt category |
| difficulty | VARCHAR(50) | Difficulty level |
| time_limit_seconds | INTEGER | Time limit |
| started_at | TIMESTAMP | Start time |
| ended_at | TIMESTAMP | End time |

#### `player_responses`
Player submissions for each round.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| round_id | UUID/TEXT | References game_rounds.id |
| player_id | UUID/TEXT | References game_players.id |
| response | TEXT | Player's response |
| response_time_ms | INTEGER | Response time |
| is_ai_generated | BOOLEAN | AI or human |
| creativity_score | FLOAT | AI-judged creativity |
| humor_score | FLOAT | AI-judged humor |
| relevance_score | FLOAT | AI-judged relevance |
| total_score | FLOAT | Combined score |
| submitted_at | TIMESTAMP | Submission time |

#### `votes`
Player votes on responses.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| response_id | UUID/TEXT | References player_responses.id |
| voter_id | UUID/TEXT | References game_players.id |
| vote_type | VARCHAR(50) | funny/creative/best |
| created_at | TIMESTAMP | Vote time |

### 4. Analytics Tables

#### `game_stats`
Aggregated game statistics.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| game_id | UUID/TEXT | References games.id |
| total_responses | INTEGER | Total responses |
| ai_responses | INTEGER | AI-generated count |
| human_responses | INTEGER | Human count |
| avg_response_time_ms | FLOAT | Average response time |
| avg_creativity_score | FLOAT | Average creativity |
| avg_humor_score | FLOAT | Average humor |
| avg_relevance_score | FLOAT | Average relevance |

#### `leaderboard`
Global player rankings.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| user_id | UUID/TEXT | References users.id |
| total_games_played | INTEGER | Games played |
| total_wins | INTEGER | Games won |
| total_score | INTEGER | Cumulative score |
| avg_score_per_game | FLOAT | Average score |
| rank | INTEGER | Global rank |

#### `achievements`
Available achievements.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID/TEXT | Primary key |
| code | VARCHAR(100) | Unique code |
| name | VARCHAR(255) | Display name |
| description | TEXT | Description |
| points | INTEGER | Point value |
| category | VARCHAR(100) | Achievement category |

## 🔄 Migrations

### Running Migrations

```bash
# PostgreSQL
./layers/L3_operational/scripts/run-migrations.sh --type postgres

# SQLite
./layers/L3_operational/scripts/run-migrations.sh --type sqlite

# Custom database URL
./layers/L3_operational/scripts/run-migrations.sh \
  --type postgres \
  --url "postgresql://user:pass@host:5432/dbname"
```

### Migration Files

Migrations are located in:
- PostgreSQL: `layers/L3_operational/architecture/server/migrations/postgres/`
- SQLite: `layers/L3_operational/architecture/server/migrations/sqlite/`

### Creating New Migrations

1. Create a new SQL file with incrementing number:
   ```
   004_your_migration_name.sql
   ```

2. Write migration SQL:
   ```sql
   -- PostgreSQL example
   CREATE TABLE new_table (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name VARCHAR(255) NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   ```

3. Create equivalent SQLite version
4. Test migrations: `./scripts/test-migrations.sh`

## 🔧 Database Configuration

### Environment Variables

```bash
# Database type (postgres or sqlite)
DATABASE_TYPE=postgres

# Connection URL
DATABASE_URL=postgresql://user:pass@localhost:5432/hal9

# Connection pool settings
DATABASE_POOL_SIZE=50
DATABASE_MAX_CONNECTIONS=200
DATABASE_SSL_MODE=require
```

### Connection Pool Settings

| Setting | Development | Production |
|---------|-------------|------------|
| Pool Size | 10 | 50-100 |
| Max Connections | 100 | 200-500 |
| Connection Timeout | 30s | 30s |
| Idle Timeout | 10m | 10m |
| Max Lifetime | 30m | 30m |

## 🔒 Security Considerations

1. **Passwords**: Always use bcrypt with cost factor 12+
2. **API Keys**: Store only hashed versions
3. **Sessions**: Implement expiration and rotation
4. **SQL Injection**: Use parameterized queries only
5. **Permissions**: Implement row-level security where needed

## 📈 Performance Optimization

### Indexes
All foreign keys and frequently queried columns are indexed.

### Partitioning (PostgreSQL)
The `signals` table is partitioned by month for better performance.

### Connection Pooling
Use connection pools to avoid connection overhead.

### Query Optimization
- Use prepared statements
- Batch inserts for bulk operations
- Implement pagination for large result sets

## 🔍 Monitoring

### Key Metrics to Monitor
- Connection pool utilization
- Query execution time
- Table sizes and growth rate
- Index usage statistics
- Slow query log

### Health Checks
```sql
-- Check connection
SELECT 1;

-- Check table sizes
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

## 🚨 Troubleshooting

### Common Issues

1. **Migration Failures**
   - Check for existing objects
   - Verify permissions
   - Review migration order

2. **Connection Pool Exhaustion**
   - Increase pool size
   - Check for connection leaks
   - Review query performance

3. **Slow Queries**
   - Check EXPLAIN plans
   - Add missing indexes
   - Optimize query structure

### Useful Commands

```bash
# PostgreSQL
psql -d hal9 -c "SELECT version();"
psql -d hal9 -c "\d+"
psql -d hal9 -c "SELECT * FROM pg_stat_activity;"

# SQLite
sqlite3 hal9.db ".tables"
sqlite3 hal9.db ".schema games"
sqlite3 hal9.db "PRAGMA integrity_check;"
```