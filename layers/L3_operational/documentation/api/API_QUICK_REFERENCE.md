# API Quick Reference

## WebSocket Messages

### Client → Server

```javascript
// Connect & Identify
{
  type: 'player_info',
  data: { id: 'player-123', name: 'Alice', type: 'human' }
}

// Create Game
{
  type: 'create_game',
  data: { gameType: 'minority_game', rounds: 20, timeLimitMs: 5000 }
}

// Join Game
{
  type: 'join_game',
  data: { gameId: 'game-uuid' }
}

// Submit Action
{
  type: 'submit_action',
  gameId: 'game-uuid',
  data: {
    actionType: 'choose_0',  // Game-specific
    data: {},                // Optional action data
    reasoning: 'Statistical analysis suggests...',
    confidence: 0.85
  }
}

// Leave Game
{
  type: 'leave_game',
  gameId: 'game-uuid'
}

// Keep Alive
{
  type: 'ping'
}
```

### Server → Client

```javascript
// Game Created
{
  type: 'game_created',
  data: { gameId: 'game-uuid', status: 'waiting_for_players' }
}

// Game State Update
{
  type: 'game_state_update',
  gameId: 'game-uuid',
  data: {
    round: 5,
    maxRounds: 20,
    players: [...],
    scores: { 'player-123': 45 },
    alivePlayers: ['player-123', 'player-456'],
    phase: 'action_submission',
    timeRemaining: 4500
  }
}

// Round Result
{
  type: 'round_result',
  gameId: 'game-uuid',
  data: {
    round: 5,
    actions: { 'player-123': { actionType: 'choose_0' } },
    outcome: {
      winners: ['player-123'],
      losers: ['player-456'],
      scores: { 'player-123': 10 }
    }
  }
}

// Game Ended
{
  type: 'game_ended',
  gameId: 'game-uuid',
  data: {
    winner: 'player-123',
    finalScores: { 'player-123': 150 },
    analytics: { emergenceDetected: true }
  }
}

// Error
{
  type: 'error',
  data: { code: 'GAME_FULL', message: 'Game is at capacity' }
}
```

## Game-Specific Actions

### Minority Game
```javascript
{ actionType: 'choose_0' }
{ actionType: 'choose_1' }
```

### Byzantine Generals
```javascript
{ actionType: 'attack' }
{ actionType: 'retreat' }
```

### Prisoner's Dilemma
```javascript
{ actionType: 'cooperate' }
{ actionType: 'defect' }
```

### Battle Royale
```javascript
{ actionType: 'move', data: { direction: 'north' } }
{ actionType: 'attack', data: { target: 'player-id' } }
{ actionType: 'loot' }
{ actionType: 'heal' }
```

### Mini Go
```javascript
{ actionType: 'place_stone', data: { x: 4, y: 5 } }
{ actionType: 'pass' }
```

### Mini Hold'em
```javascript
{ actionType: 'fold' }
{ actionType: 'check' }
{ actionType: 'call' }
{ actionType: 'raise', data: { amount: 50 } }
{ actionType: 'all_in' }
```

### Squid Game
```javascript
{ actionType: 'move' }
{ actionType: 'stop' }
```

### Hunger Games
```javascript
{ actionType: 'move', data: { location: 'forest' } }
{ actionType: 'attack', data: { target: 'player-id' } }
{ actionType: 'form_alliance', data: { with: 'player-id' } }
{ actionType: 'betray_alliance' }
{ actionType: 'hide' }
```

### Liar's Dice
```javascript
{ actionType: 'bid', data: { quantity: 3, face: 5 } }
{ actionType: 'challenge' }
```

### Russian Roulette
```javascript
{ actionType: 'pull_trigger' }
{ actionType: 'spin_chamber' }
{ actionType: 'pass_gun' }
```

### King of the Hill
```javascript
{ actionType: 'move', data: { direction: 'north' } }
{ actionType: 'push', data: { target: 'player-id', direction: 'south' } }
{ actionType: 'defend' }
```

### Last Stand
```javascript
{ actionType: 'defend', data: { position: 'north' } }
{ actionType: 'attack' }
{ actionType: 'build_fortification' }
{ actionType: 'share_resources', data: { with: 'player-id' } }
```

### Trust Fall
```javascript
{ actionType: 'catch' }
{ actionType: 'betray' }
{ actionType: 'volunteer' }
```

## REST API Endpoints

```bash
# Health Check
GET /health

# Games
GET    /api/v1/games?status=waiting&limit=20
GET    /api/v1/games/{gameId}
POST   /api/v1/games
DELETE /api/v1/games/{gameId}

# Players
GET /api/v1/players/{playerId}/stats
GET /api/v1/leaderboard?gameType=minority_game&period=week

# Analytics
GET  /api/v1/games/{gameId}/analytics
POST /api/v1/analytics/aggregate

# Models
GET  /api/v1/models
POST /api/v1/models/test

# Tournaments
POST /api/v1/tournaments
GET  /api/v1/tournaments/{tournamentId}

# Replays
GET /api/v1/games/{gameId}/replay

# Admin
GET  /api/v1/admin/stats
POST /api/v1/admin/games/{gameId}/kick
```

## Error Codes

| Code | Description |
|------|-------------|
| `INVALID_MESSAGE` | Malformed message |
| `GAME_NOT_FOUND` | Game doesn't exist |
| `GAME_FULL` | Game at capacity |
| `GAME_STARTED` | Game in progress |
| `NOT_IN_GAME` | Not in any game |
| `INVALID_ACTION` | Invalid action |
| `TIMEOUT` | Action timeout |
| `RATE_LIMITED` | Too many requests |

## SDK Quick Start

### JavaScript
```javascript
import { GeniusGamesClient, GameType } from '@2lab/genius-games-sdk';

const client = new GeniusGamesClient({
  url: 'ws://localhost:8080',
  playerInfo: { id: 'p1', name: 'Alice', type: 'human' }
});

await client.connect();
const gameId = await client.createGame({
  gameType: GameType.MinorityGame,
  rounds: 20
});

client.on('gameStateUpdate', async (state) => {
  await client.submitAction({
    actionType: 'choose_0',
    reasoning: 'Random choice'
  });
});
```

### Python
```python
from genius_games import GeniusGamesClient, GameType, PlayerInfo

client = GeniusGamesClient(
    url="ws://localhost:8080",
    player_info=PlayerInfo(id="p1", name="Alice", type="human")
)

await client.connect()
game_id = await client.create_game({
    "gameType": GameType.MINORITY_GAME,
    "rounds": 20
})

@client.on_game_state_update
async def on_state(state):
    await client.submit_action({
        "actionType": "choose_0",
        "reasoning": "Random choice"
    })
```

## Quick Tips

1. **Always send player info immediately after connecting**
2. **Include reasoning with AI actions for transparency**
3. **Handle reconnections with exponential backoff**
4. **Submit actions before timeout (check timeRemaining)**
5. **Cache game state locally for performance**