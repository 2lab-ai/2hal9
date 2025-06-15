# Genius Games JavaScript/TypeScript SDK

Official JavaScript/TypeScript SDK for the Genius Game Server - an AI collective intelligence gaming platform.

## Installation

```bash
npm install @2lab/genius-games-sdk
# or
yarn add @2lab/genius-games-sdk
```

## Quick Start

### Basic Connection

```typescript
import { GeniusGamesClient, PlayerInfo } from '@2lab/genius-games-sdk';

const client = new GeniusGamesClient({
  url: 'http://localhost:8080',
  wsUrl: 'ws://localhost:8081/ws',
  debug: true
}, {
  id: 'player1',
  name: 'Alice',
  type: 'human'
});

// Connect to server
await client.connect();

// Create a game
const gameId = await client.createGame({
  gameType: GameType.MinorityGame,
  rounds: 20,
  timeLimitMs: 5000
});

// Submit an action
await client.submitAction({
  actionType: 'choose_0',
  data: {},
  reasoning: 'Going with my gut',
  confidence: 0.7
});
```

### AI Player

```typescript
import { AIPlayer, RandomDecisionMaker } from '@2lab/genius-games-sdk';

const aiPlayer = new AIPlayer(
  { url: 'http://localhost:8080' },
  { id: 'ai1', name: 'Bot-1', type: 'ai' },
  new RandomDecisionMaker()
);

await aiPlayer.connect();
await aiPlayer.createAndPlay({
  gameType: GameType.PrisonersDilemma,
  rounds: 10,
  timeLimitMs: 3000
});
```

### AI Swarm

```typescript
import { createAISwarm, GameType } from '@2lab/genius-games-sdk';

// Create a swarm of 16 AI players
const swarm = await createAISwarm(
  { url: 'http://localhost:8080' },
  {
    size: 16,
    namePrefix: 'Swarm',
    decisionMaker: 'strategic'
  }
);

// Play a game
await swarm.createAndPlayGame({
  gameType: GameType.ByzantineGenerals,
  rounds: 15,
  timeLimitMs: 5000
});
```

## Game Types

The SDK supports all game types from the server:

### Classic Games
- `MinorityGame` - Players try to be in the minority
- `ByzantineGenerals` - Coordinate attack/retreat with potential traitors
- `PrisonersDilemma` - Cooperate or defect
- `QuantumConsensus` - Quantum-inspired consensus mechanics

### Collective Intelligence Games
- `CollectiveMaze` - Navigate a maze together
- `RecursiveReasoning` - Multi-level thinking challenges
- `SwarmOptimization` - Particle swarm optimization

### Strategic Games
- `MiniGo` - 9x9 Go board
- `MiniHoldem` - Simplified Texas Hold'em

### Death Games
- `SquidGame` - Red light, green light
- `BattleRoyale` - Last player standing
- `HungerGames` - Survival with alliances
- `LiarsDice` - Bluffing dice game
- `RussianRoulette` - Luck-based elimination
- `KingOfTheHill` - Control the center
- `LastStand` - Wave survival
- `TrustFall` - Trust and betrayal

## Event Handling

```typescript
client.setHandlers({
  onConnect: () => console.log('Connected!'),
  onDisconnect: (reason) => console.log('Disconnected:', reason),
  onGameCreated: (gameId) => console.log('Game created:', gameId),
  onGameJoined: (gameState) => console.log('Joined game:', gameState),
  onGameStateUpdate: (gameState) => {
    console.log('Round:', gameState.round);
    console.log('Scores:', gameState.scores);
  },
  onRoundResult: (result) => {
    console.log('Round outcome:', result.outcome);
    if (result.outcome.emergenceDetected) {
      console.log('Emergence detected!');
    }
  },
  onGameEnded: (result) => {
    console.log('Winner:', result.winner);
    console.log('Final scores:', result.finalScores);
  },
  onError: (error) => console.error('Error:', error)
});
```

## Custom AI Decision Makers

```typescript
import { AIDecisionMaker, GameState, GameType, Action } from '@2lab/genius-games-sdk';

class MyCustomAI implements AIDecisionMaker {
  async makeDecision(gameState: GameState, gameType: GameType): Promise<Action> {
    // Analyze game state
    const myScore = gameState.scores[this.getName()] || 0;
    const round = gameState.round;
    
    // Make decision based on game type
    let actionType = 'default';
    let reasoning = 'Custom logic';
    
    if (gameType === GameType.MinorityGame) {
      // Analyze history to find patterns
      const recentChoices = this.analyzeHistory(gameState.history);
      actionType = recentChoices.zeros > recentChoices.ones ? 'choose_1' : 'choose_0';
      reasoning = `Choosing minority based on ${recentChoices}`;
    }
    
    return {
      playerId: '',
      actionType,
      data: {},
      reasoning,
      confidence: 0.8
    };
  }
  
  getName(): string {
    return 'MyCustomAI';
  }
  
  private analyzeHistory(history: any[]): any {
    // Custom analysis logic
    return { zeros: 0, ones: 0 };
  }
}
```

## Simulation

```typescript
import { GameSimulator, GameType } from '@2lab/genius-games-sdk';

const simulator = new GameSimulator({ url: 'http://localhost:8080' });

// Run automated simulation
const results = await simulator.runSimulation({
  games: [
    { gameType: GameType.MinorityGame, rounds: 20, timeLimitMs: 5000 },
    { gameType: GameType.PrisonersDilemma, rounds: 10, timeLimitMs: 3000 }
  ],
  swarmSize: 8,
  swarmType: 'mixed',
  rounds: 100,
  verbose: true
});

console.log('Simulation Results:');
console.log('Win rates:', results.winRates);
console.log('Average scores:', results.avgScores);
console.log('Emergence events:', results.emergenceEvents);
```

## TypeScript Support

The SDK is written in TypeScript and provides full type definitions:

```typescript
import { 
  GameType,
  GameState,
  Action,
  RoundResult,
  GameResult,
  PlayerInfo,
  ConnectionOptions 
} from '@2lab/genius-games-sdk';
```

## Browser Support

The SDK works in both Node.js and browser environments:

```html
<script src="https://unpkg.com/@2lab/genius-games-sdk/dist/index.umd.js"></script>
<script>
  const { GeniusGamesClient } = window.GeniusGames;
  
  const client = new GeniusGamesClient({
    url: 'http://localhost:8080'
  }, {
    id: 'browser-player',
    name: 'Browser Player',
    type: 'human'
  });
</script>
```

## Examples

Check the `examples/` directory for more detailed examples:

- `examples/basic-game.js` - Simple game creation and playing
- `examples/ai-swarm.js` - Running an AI swarm
- `examples/custom-ai.js` - Implementing custom AI logic
- `examples/tournament.js` - Running a tournament
- `examples/browser/` - Browser-based examples

## API Reference

### GeniusGamesClient

Main client for connecting to the game server.

**Constructor**
```typescript
new GeniusGamesClient(options: ConnectionOptions, playerInfo: PlayerInfo)
```

**Methods**
- `connect(): Promise<void>` - Connect to server
- `disconnect(): void` - Disconnect from server
- `createGame(config: GameConfig): Promise<string>` - Create a new game
- `joinGame(gameId: string): Promise<GameState>` - Join existing game
- `submitAction(action: Action): Promise<void>` - Submit game action
- `leaveGame(): Promise<void>` - Leave current game
- `setHandlers(handlers: EventHandlers): void` - Set event handlers

### AIPlayer

Automated player that can play games using an AI decision maker.

**Constructor**
```typescript
new AIPlayer(
  connectionOptions: ConnectionOptions,
  playerInfo: PlayerInfo,
  decisionMaker: AIDecisionMaker
)
```

**Methods**
- `connect(): Promise<void>` - Connect to server
- `disconnect(): void` - Disconnect from server
- `createAndPlay(config: GameConfig): Promise<void>` - Create and play a game
- `joinAndPlay(gameId: string): Promise<void>` - Join and play a game
- `stopPlaying(): void` - Stop playing current game

### AISwarm

Manage multiple AI players as a collective.

**Methods**
- `connect(): Promise<void>` - Connect all players
- `disconnect(): void` - Disconnect all players
- `createAndPlayGame(config: GameConfig): Promise<void>` - Create game with swarm
- `joinGame(gameId: string): Promise<void>` - Join game with swarm
- `getStats(): SwarmStats` - Get swarm statistics

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) in the main repository.

## License

MIT License - see [LICENSE](../../LICENSE) in the main repository.