# HAL9 API Reference

## 📡 REST API Endpoints

### Base URL
- Local: `http://localhost:3456`
- Production: `https://api.hal9.ai` (coming soon)

### Authentication
현재는 인증이 필요하지 않습니다. 프로덕션에서는 JWT 토큰을 사용할 예정입니다.

## 🎮 AI Genius Game API

### Create Game
```http
POST /api/games/create
Content-Type: application/json

{
  "game_type": {
    "type": "ConsciousnessEmergence"
  },
  "max_rounds": 20
}

Response:
{
  "id": "uuid",
  "status": "created"
}
```

### List Games
```http
GET /api/games

Response:
[
  {
    "id": "uuid",
    "status": "Waiting"
  }
]
```

### Get Game State
```http
GET /api/games/{game_id}

Response:
{
  "id": "uuid",
  "game_type": { "type": "ConsciousnessEmergence" },
  "status": "Running",
  "round": 5,
  "max_rounds": 20,
  "players": {},
  "board": [[...]],
  "consciousness_level": 0.75,
  "events": [...],
  "winner": null
}
```

## 🔌 WebSocket API

### Connection
```javascript
const ws = new WebSocket('ws://localhost:3456/ws/{game_id}');
```

### Message Types

#### Join Game
```json
{
  "type": "join_game",
  "player_name": "Player 1",
  "player_type": {
    "type": "hal9_collective",
    "agent_count": 6
  }
}
```

#### Game Action
```json
{
  "type": "game_action",
  "action": {
    "action": "place_neuron",
    "x": 10,
    "y": 10
  }
}
```

#### Game State Update
```json
{
  "type": "game_state",
  "state": {
    // Full game state object
  }
}
```

## 🧠 Consciousness API (Coming Soon)

### Calculate Phi
```http
POST /api/consciousness/phi
Content-Type: application/json

{
  "neurons": [...],
  "connections": [...]
}

Response:
{
  "phi": 1.618,
  "emergence_detected": true,
  "compression_ratio": 2.718
}
```

### Monitor Emergence
```http
GET /api/consciousness/monitor/{session_id}

Response (Server-Sent Events):
data: {"phi": 1.2, "timestamp": "2025-06-18T12:00:00Z"}
data: {"phi": 1.5, "timestamp": "2025-06-18T12:00:01Z"}
data: {"phi": 1.618, "timestamp": "2025-06-18T12:00:02Z", "emergence": true}
```

## 🤖 Self-Organization API (Coming Soon)

### Create Neuron Pool
```http
POST /api/neurons/pool
Content-Type: application/json

{
  "count": 1000,
  "initial_state": "random"
}

Response:
{
  "pool_id": "uuid",
  "neurons": 1000,
  "status": "initializing"
}
```

### Trigger Self-Organization
```http
POST /api/neurons/pool/{pool_id}/organize

Response:
{
  "layers_formed": 6,
  "time_microseconds": 845,
  "compression_boundaries": [1.0, 1.414, 1.618, 2.0, 2.718, 3.14]
}
```

## 📊 Metrics API

### System Metrics
```http
GET /api/metrics

Response:
{
  "active_neurons": 50000,
  "total_connections": 2500000,
  "ops_per_second": 847000,
  "memory_usage_mb": 512,
  "uptime_seconds": 3600
}
```

### Performance History
```http
GET /api/metrics/history?period=1h

Response:
{
  "timestamps": [...],
  "fps": [...],
  "ops_per_second": [...],
  "phi_values": [...]
}
```

## 🔧 Admin API (Protected)

### System Health
```http
GET /api/admin/health
Authorization: Bearer {token}

Response:
{
  "status": "healthy",
  "components": {
    "database": "ok",
    "redis": "ok",
    "neurons": "ok"
  }
}
```

### Reset System
```http
POST /api/admin/reset
Authorization: Bearer {token}

Response:
{
  "status": "reset_complete",
  "timestamp": "2025-06-18T12:00:00Z"
}
```

## 📝 SDK Usage

### JavaScript/TypeScript
```javascript
import { HAL9Client } from '@2lab/hal9-sdk';

const client = new HAL9Client({
  baseURL: 'http://localhost:3456'
});

// Create game
const game = await client.games.create({
  gameType: 'ConsciousnessEmergence',
  maxRounds: 20
});

// Monitor consciousness
client.consciousness.monitor(sessionId, (data) => {
  console.log(`Phi: ${data.phi}`);
  if (data.emergence) {
    console.log('Consciousness emerged!');
  }
});
```

### Python
```python
from hal9 import HAL9Client

client = HAL9Client(base_url="http://localhost:3456")

# Create neuron pool
pool = client.neurons.create_pool(count=1000)

# Trigger self-organization
result = client.neurons.organize(pool.id)
print(f"Layers formed: {result.layers_formed}")
print(f"Time: {result.time_microseconds}μs")
```

## 🚨 Error Codes

| Code | Description |
|------|-------------|
| 400 | Bad Request - Invalid parameters |
| 404 | Not Found - Resource doesn't exist |
| 409 | Conflict - Game already started |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error |
| 503 | Service Unavailable - System overloaded |

## 📈 Rate Limits

- REST API: 100 requests/minute
- WebSocket: 10 messages/second
- Consciousness calculations: 10 requests/minute

## 🔐 Security Notes

1. **CORS**: Enabled for all origins in development
2. **WSS**: Use secure WebSocket in production
3. **API Keys**: Required for production use
4. **Rate Limiting**: Implemented per IP address

## 📚 Further Reading

- [WebSocket Protocol Details](./websocket-protocol.md)
- [Consciousness Metrics Explained](./consciousness-metrics.md)
- [SDK Development Guide](./sdk-guide.md)