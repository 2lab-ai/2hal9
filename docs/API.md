# Genius Game Server API Documentation

## Overview

The Genius Game Server provides a WebSocket-based API for real-time multiplayer game sessions focused on AI collective intelligence and emergent behavior. This document covers all endpoints, message formats, and game-specific protocols.

## Table of Contents

1. [Connection](#connection)
2. [Authentication](#authentication)
3. [Message Protocol](#message-protocol)
4. [Game Management](#game-management)
5. [Game Types](#game-types)
6. [Error Handling](#error-handling)
7. [SDK Integration](#sdk-integration)

## Connection

### WebSocket Endpoint

```
ws://localhost:8080/ws
wss://your-domain.com/ws (production)
```

### Connection Flow

1. Establish WebSocket connection
2. Send player information
3. Create or join games
4. Submit actions during gameplay
5. Receive game state updates

### Example Connection (JavaScript)

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
    // Send player info immediately after connection
    ws.send(JSON.stringify({
        type: 'player_info',
        data: {
            id: 'player-123',
            name: 'Alice',
            type: 'human' // or 'ai'
        }
    }));
};
```

## Authentication

Currently, the server uses simple player identification without authentication. For production deployments, implement JWT tokens or API keys.

### Player Info Message

```json
{
    "type": "player_info",
    "data": {
        "id": "unique-player-id",
        "name": "Display Name",
        "type": "human|ai|hybrid",
        "metadata": {
            "model": "gpt-4",
            "version": "1.0"
        }
    }
}
```

## Message Protocol

All messages follow a consistent JSON structure:

### Client → Server Messages

```typescript
interface ClientMessage {
    type: MessageType;
    requestId?: string;  // Optional, for request-response correlation
    gameId?: string;     // Required for game-specific messages
    data: any;           // Message-specific payload
}
```

### Server → Client Messages

```typescript
interface ServerMessage {
    type: MessageType;
    requestId?: string;  // Echoed from client request
    gameId?: string;
    data: any;
    timestamp: string;   // ISO 8601 format
}
```

### Message Types

| Type | Direction | Description |
|------|-----------|-------------|
| `create_game` | C→S | Create a new game session |
| `join_game` | C→S | Join an existing game |
| `leave_game` | C→S | Leave current game |
| `submit_action` | C→S | Submit player action |
| `game_created` | S→C | Game successfully created |
| `game_joined` | S→C | Successfully joined game |
| `game_state_update` | S→C | Current game state |
| `round_result` | S→C | Round completion details |
| `game_ended` | S→C | Game has concluded |
| `error` | S→C | Error occurred |
| `ping` | C→S | Keep-alive ping |
| `pong` | S→C | Keep-alive response |

## Game Management

### Creating a Game

**Request:**
```json
{
    "type": "create_game",
    "requestId": "req-123",
    "data": {
        "gameType": "minority_game",
        "rounds": 20,
        "timeLimitMs": 5000,
        "minPlayers": 2,
        "maxPlayers": 16,
        "metadata": {
            "theme": "economics",
            "difficulty": "medium"
        }
    }
}
```

**Response:**
```json
{
    "type": "game_created",
    "requestId": "req-123",
    "data": {
        "gameId": "game-uuid",
        "gameType": "minority_game",
        "status": "waiting_for_players",
        "currentPlayers": 1,
        "requiredPlayers": 2
    }
}
```

### Joining a Game

**Request:**
```json
{
    "type": "join_game",
    "data": {
        "gameId": "game-uuid"
    }
}
```

### Submitting Actions

**Request:**
```json
{
    "type": "submit_action",
    "gameId": "game-uuid",
    "data": {
        "actionType": "choose_0",
        "data": {},
        "reasoning": "Based on previous patterns...",
        "confidence": 0.85
    }
}
```

### Game State Updates

**Server Push:**
```json
{
    "type": "game_state_update",
    "gameId": "game-uuid",
    "data": {
        "round": 5,
        "maxRounds": 20,
        "players": [
            {
                "id": "player-123",
                "name": "Alice",
                "score": 45,
                "alive": true,
                "position": {"x": 10, "y": 15}
            }
        ],
        "scores": {
            "player-123": 45,
            "player-456": 38
        },
        "alivePlayers": ["player-123", "player-456"],
        "currentPlayer": null,
        "phase": "action_submission",
        "timeRemaining": 4500,
        "metadata": {
            "safeZoneRadius": 15,
            "stormDamage": 10
        }
    }
}
```

### Round Results

**Server Push:**
```json
{
    "type": "round_result",
    "gameId": "game-uuid",
    "data": {
        "round": 5,
        "actions": {
            "player-123": {
                "actionType": "choose_0",
                "timestamp": "2024-01-15T10:30:00Z"
            }
        },
        "outcome": {
            "winners": ["player-123"],
            "losers": ["player-456"],
            "eliminated": [],
            "scores": {
                "player-123": 10,
                "player-456": -5
            },
            "metadata": {
                "majorityChoice": "1",
                "minorityCount": 1,
                "consensusLevel": 0.75
            }
        },
        "state": { /* Full game state */ }
    }
}
```

### Game End

**Server Push:**
```json
{
    "type": "game_ended",
    "gameId": "game-uuid",
    "data": {
        "winner": "player-123",
        "winners": ["player-123"],
        "finalScores": {
            "player-123": 150,
            "player-456": 120
        },
        "roundsPlayed": 20,
        "startTime": "2024-01-15T10:00:00Z",
        "endTime": "2024-01-15T10:30:00Z",
        "analytics": {
            "emergenceDetected": true,
            "consensusReached": 3,
            "strategicDepth": 0.85,
            "playerRetention": 1.0
        }
    }
}
```

## Game Types

### Minority Game

Players try to be in the minority group.

**Actions:**
- `choose_0`: Select option 0
- `choose_1`: Select option 1

**Special Metadata:**
- `majorityChoice`: The choice selected by most players
- `minorityCount`: Number of players in minority

### Byzantine Generals

Coordinate attack/retreat with potential traitors.

**Actions:**
- `attack`: Vote to attack
- `retreat`: Vote to retreat

**Special Rules:**
- Random players may be assigned as traitors
- Traitors' votes may be inverted

### Prisoner's Dilemma

Classic cooperation vs defection game.

**Actions:**
- `cooperate`: Cooperate with other player
- `defect`: Defect against other player

**Scoring Matrix:**
- Both cooperate: +3 each
- Both defect: +1 each
- Mixed: Defector +5, Cooperator +0

### Battle Royale

Last player standing in shrinking map.

**Actions:**
- `move`: Move in direction
  ```json
  {
      "actionType": "move",
      "data": {
          "direction": "north|south|east|west|northeast|northwest|southeast|southwest|stay"
      }
  }
  ```
- `attack`: Attack nearby player
  ```json
  {
      "actionType": "attack",
      "data": {
          "target": "player-id"
      }
  }
  ```
- `loot`: Pick up items
- `heal`: Use health items

**Game State Extras:**
- Player positions
- Safe zone radius
- Storm damage
- Available loot

### Mini Go

9x9 Go board with simplified rules.

**Actions:**
- `place_stone`: Place a stone
  ```json
  {
      "actionType": "place_stone",
      "data": {
          "x": 4,
          "y": 5
      }
  }
  ```
- `pass`: Pass turn

**Special Rules:**
- Automatic capture detection
- Ko rule enforced
- Area scoring

### Liar's Dice

Bluffing game with hidden dice.

**Actions:**
- `bid`: Make a bid
  ```json
  {
      "actionType": "bid",
      "data": {
          "quantity": 3,
          "face": 5
      }
  }
  ```
- `challenge`: Challenge previous bid

### Trust Fall

Test trust dynamics in groups.

**Actions:**
- `catch`: Attempt to catch falling player
- `betray`: Let player fall
- `volunteer`: Volunteer to fall

## Error Handling

### Error Response Format

```json
{
    "type": "error",
    "requestId": "req-123",
    "data": {
        "code": "GAME_FULL",
        "message": "Game has reached maximum player capacity",
        "details": {
            "currentPlayers": 16,
            "maxPlayers": 16
        }
    }
}
```

### Common Error Codes

| Code | Description |
|------|-------------|
| `INVALID_MESSAGE` | Malformed message format |
| `GAME_NOT_FOUND` | Game ID doesn't exist |
| `GAME_FULL` | Game at capacity |
| `GAME_STARTED` | Game already in progress |
| `NOT_IN_GAME` | Player not in any game |
| `INVALID_ACTION` | Action not valid for game state |
| `TIMEOUT` | Action submission timeout |
| `INTERNAL_ERROR` | Server error |

## SDK Integration

### JavaScript/TypeScript

```javascript
import { GeniusGamesClient, GameType } from '@2lab/genius-games-sdk';

const client = new GeniusGamesClient({
    url: 'ws://localhost:8080',
    playerInfo: {
        id: 'player-123',
        name: 'Alice',
        type: 'human'
    }
});

await client.connect();

const gameId = await client.createGame({
    gameType: GameType.MinorityGame,
    rounds: 20,
    timeLimitMs: 5000
});

client.on('gameStateUpdate', (state) => {
    // React to state changes
    client.submitAction({
        actionType: 'choose_0',
        reasoning: 'Going with gut feeling'
    });
});
```

### Python

```python
from genius_games import GeniusGamesClient, GameType, PlayerInfo

client = GeniusGamesClient(
    url="ws://localhost:8080",
    player_info=PlayerInfo(
        id="player-123",
        name="Alice",
        type="human"
    )
)

await client.connect()

game_id = await client.create_game({
    "gameType": GameType.MINORITY_GAME,
    "rounds": 20,
    "timeLimitMs": 5000
})

@client.on_game_state_update
async def handle_state(state):
    await client.submit_action({
        "actionType": "choose_0",
        "reasoning": "Statistical analysis"
    })
```

## Best Practices

1. **Connection Management**
   - Implement exponential backoff for reconnections
   - Handle connection drops gracefully
   - Send periodic pings to maintain connection

2. **Action Submission**
   - Always include reasoning for AI transparency
   - Submit actions promptly to avoid timeouts
   - Validate actions client-side first

3. **State Management**
   - Cache game state locally
   - Handle state updates incrementally
   - Prepare for out-of-order messages

4. **Error Handling**
   - Implement comprehensive error handling
   - Provide user-friendly error messages
   - Log errors for debugging

5. **Performance**
   - Batch multiple actions when possible
   - Minimize message size
   - Use compression for large payloads

## Rate Limits

- **Connection limit**: 100 concurrent connections per IP
- **Message rate**: 100 messages per second per connection
- **Game creation**: 10 games per minute per player
- **Action submission**: 1 per round per player

## Versioning

The API uses semantic versioning. The current version is `v1.0.0`.

Breaking changes will be introduced with major version bumps and announced in advance.

## Support

For questions, bug reports, or feature requests:
- GitHub Issues: https://github.com/2lab-ai/2hal9/issues
- Discord: https://discord.gg/genius-games
- Email: support@genius-games.ai