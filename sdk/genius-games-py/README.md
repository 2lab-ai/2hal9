# Genius Games Python SDK

Official Python SDK for the Genius Game Server - an AI collective intelligence gaming platform.

## Installation

```bash
pip install genius-games-sdk
```

## Quick Start

```python
import asyncio
from genius_games import GeniusGamesClient, GameType, PlayerInfo

async def main():
    # Create client
    client = GeniusGamesClient(
        url="http://localhost:8080",
        player_info=PlayerInfo(
            id="player1",
            name="Alice",
            type="human"
        )
    )
    
    # Connect to server
    await client.connect()
    
    # Create a game
    game_id = await client.create_game({
        "gameType": GameType.MINORITY_GAME,
        "rounds": 20,
        "timeLimitMs": 5000
    })
    
    # Submit an action
    await client.submit_action({
        "actionType": "choose_0",
        "data": {},
        "reasoning": "Going with my gut",
        "confidence": 0.7
    })
    
    # Disconnect when done
    await client.disconnect()

asyncio.run(main())
```

## AI Player Example

```python
from genius_games import AIPlayer, RandomDecisionMaker

async def run_ai_player():
    ai = AIPlayer(
        url="http://localhost:8080",
        player_info=PlayerInfo(id="ai1", name="Bot-1", type="ai"),
        decision_maker=RandomDecisionMaker()
    )
    
    await ai.connect()
    await ai.create_and_play({
        "gameType": GameType.PRISONERS_DILEMMA,
        "rounds": 10,
        "timeLimitMs": 3000
    })

asyncio.run(run_ai_player())
```

## AI Swarm Example

```python
from genius_games import create_ai_swarm, GameType

async def run_swarm():
    # Create a swarm of 16 AI players
    swarm = await create_ai_swarm(
        url="http://localhost:8080",
        size=16,
        name_prefix="Swarm",
        decision_maker="strategic"
    )
    
    # Play a game
    await swarm.create_and_play_game({
        "gameType": GameType.BYZANTINE_GENERALS,
        "rounds": 15,
        "timeLimitMs": 5000
    })
    
    # Disconnect when done
    await swarm.disconnect()

asyncio.run(run_swarm())
```

## Custom AI Decision Maker

```python
from genius_games import AIDecisionMaker, GameState, GameType, Action

class MyCustomAI(AIDecisionMaker):
    async def make_decision(self, game_state: GameState, game_type: GameType) -> Action:
        # Analyze game state
        my_score = game_state.scores.get(self.get_name(), 0)
        round_num = game_state.round
        
        # Make decision based on game type
        if game_type == GameType.MINORITY_GAME:
            # Simple anti-majority strategy
            action_type = "choose_0" if round_num % 2 == 0 else "choose_1"
            reasoning = f"Alternating strategy at round {round_num}"
        else:
            action_type = "default"
            reasoning = "No specific strategy"
        
        return Action(
            action_type=action_type,
            data={},
            reasoning=reasoning,
            confidence=0.7
        )
    
    def get_name(self) -> str:
        return "MyCustomAI"
```

## Event Handling

```python
# Set event handlers
client.on_connect = lambda: print("Connected!")
client.on_disconnect = lambda reason: print(f"Disconnected: {reason}")
client.on_game_created = lambda game_id: print(f"Game created: {game_id}")
client.on_game_state_update = lambda state: print(f"Round {state.round}")
client.on_round_result = lambda result: print(f"Winners: {result.outcome.winners}")
client.on_game_ended = lambda result: print(f"Winner: {result.winner}")
client.on_error = lambda error: print(f"Error: {error}")
```

## Supported Game Types

- `MINORITY_GAME` - Players try to be in the minority
- `BYZANTINE_GENERALS` - Coordinate with potential traitors
- `PRISONERS_DILEMMA` - Cooperate or defect
- `QUANTUM_CONSENSUS` - Quantum-inspired consensus
- `COLLECTIVE_MAZE` - Navigate together
- `RECURSIVE_REASONING` - Multi-level thinking
- `SWARM_OPTIMIZATION` - Particle swarm optimization
- `MINI_GO` - 9x9 Go board
- `MINI_HOLDEM` - Texas Hold'em
- `SQUID_GAME` - Red light, green light
- `BATTLE_ROYALE` - Last player standing
- `HUNGER_GAMES` - Survival with alliances
- `LIARS_DICE` - Bluffing dice game
- `RUSSIAN_ROULETTE` - Luck-based elimination
- `KING_OF_THE_HILL` - Control the center
- `LAST_STAND` - Wave survival
- `TRUST_FALL` - Trust and betrayal

## Running Simulations

```python
from genius_games import GameSimulator, GameType

async def run_simulation():
    simulator = GameSimulator("http://localhost:8080")
    
    results = await simulator.run_simulation(
        games=[
            {"gameType": GameType.MINORITY_GAME, "rounds": 20, "timeLimitMs": 5000},
            {"gameType": GameType.PRISONERS_DILEMMA, "rounds": 10, "timeLimitMs": 3000}
        ],
        swarm_size=8,
        swarm_type="mixed",
        rounds=100,
        verbose=True
    )
    
    print(f"Win rates: {results.win_rates}")
    print(f"Average scores: {results.avg_scores}")
    print(f"Emergence events: {results.emergence_events}")

asyncio.run(run_simulation())
```

## Requirements

- Python 3.8+
- asyncio
- websocket-client
- aiohttp

## License

MIT License - see LICENSE file for details.