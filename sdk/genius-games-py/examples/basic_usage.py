#!/usr/bin/env python3
"""
Basic usage example for Genius Games Python SDK
"""

import asyncio
import logging
from genius_games import GeniusGamesClient, GameType, PlayerInfo, Action

# Set up logging
logging.basicConfig(level=logging.INFO)


async def basic_example():
    """Basic example of connecting and playing a game"""
    
    # Create client
    client = GeniusGamesClient(
        url="http://localhost:8080",
        player_info=PlayerInfo(
            id="player1",
            name="Alice",
            type="human"
        ),
        debug=True
    )
    
    # Set up event handlers
    client.on_connect = lambda: print("✅ Connected to server!")
    client.on_disconnect = lambda reason: print(f"👋 Disconnected: {reason}")
    client.on_game_created = lambda game_id: print(f"🎮 Game created: {game_id}")
    client.on_game_ended = lambda result: print(f"🏆 Winner: {result.winner}")
    
    def on_state_update(state):
        print(f"📊 Round {state.round}/{state.max_rounds}")
        print(f"   Players alive: {len(state.alive_players)}")
        print(f"   Your score: {state.scores.get('Alice', 0)}")
    
    client.on_game_state_update = on_state_update
    
    def on_round_result(result):
        print(f"✅ Round {result.round} complete")
        print(f"   Winners: {result.outcome.winners}")
        if result.outcome.eliminated:
            print(f"   Eliminated: {result.outcome.eliminated}")
    
    client.on_round_result = on_round_result
    
    try:
        # Connect to server
        print("Connecting to Genius Game Server...")
        await client.connect()
        
        # Create a Minority Game
        print("\nCreating Minority Game...")
        game_id = await client.create_game({
            "gameType": GameType.MINORITY_GAME.value,
            "rounds": 10,
            "timeLimitMs": 5000
        })
        
        print(f"Game ID: {game_id}")
        print("Waiting for other players...")
        
        # Game will start automatically when enough players join
        # The client will receive state updates and we need to submit actions
        
        # Simple game loop
        round_count = 0
        while round_count < 10:
            state = client.get_current_state()
            if state and state.round > round_count:
                round_count = state.round
                
                # Make a decision (alternating strategy)
                choice = "choose_0" if round_count % 2 == 0 else "choose_1"
                
                print(f"\n🤔 Making decision for round {round_count}: {choice}")
                
                await client.submit_action(Action(
                    action_type=choice,
                    reasoning="Alternating strategy",
                    confidence=0.7
                ))
            
            await asyncio.sleep(0.5)
        
        # Wait a bit for final results
        await asyncio.sleep(2)
        
    except Exception as e:
        print(f"❌ Error: {e}")
    
    finally:
        # Disconnect
        print("\nDisconnecting...")
        await client.disconnect()
        print("Done!")


async def manual_action_example():
    """Example with manual action input"""
    
    client = GeniusGamesClient(
        url="http://localhost:8080",
        player_info=PlayerInfo(
            id="manual_player",
            name="ManualPlayer",
            type="human"
        )
    )
    
    # Track game state
    current_round = 0
    
    def on_state_update(state):
        nonlocal current_round
        if state.round > current_round:
            current_round = state.round
            print(f"\n=== Round {current_round} ===")
            print("Choose your action:")
            print("  0 - Choose option 0")
            print("  1 - Choose option 1")
    
    client.on_game_state_update = on_state_update
    
    try:
        await client.connect()
        
        # Create Minority Game
        game_id = await client.create_game({
            "gameType": GameType.MINORITY_GAME.value,
            "rounds": 5,
            "timeLimitMs": 30000  # 30 seconds for manual input
        })
        
        print(f"Game created: {game_id}")
        print("Game will start when other players join...")
        
        # Game loop with manual input
        while current_round < 5:
            if client.get_current_state() and client.get_current_state().round > 0:
                # Get user input
                try:
                    choice = input("Your choice (0 or 1): ").strip()
                    if choice in ["0", "1"]:
                        action_type = f"choose_{choice}"
                        await client.submit_action(Action(
                            action_type=action_type,
                            reasoning="Manual choice",
                            confidence=1.0
                        ))
                        print(f"Submitted: {action_type}")
                except:
                    pass
            
            await asyncio.sleep(0.5)
        
        await asyncio.sleep(2)
        
    finally:
        await client.disconnect()


if __name__ == "__main__":
    print("🎮 Genius Games Python SDK - Basic Example\n")
    print("Choose example:")
    print("1. Automated player")
    print("2. Manual input player")
    
    choice = input("\nYour choice (1 or 2): ").strip()
    
    if choice == "2":
        asyncio.run(manual_action_example())
    else:
        asyncio.run(basic_example())