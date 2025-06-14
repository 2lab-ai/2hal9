#!/usr/bin/env python3
"""
AI Genius Game Client Demo
Demonstrates how to create players and run a game
"""

import asyncio
import aiohttp
import json
from datetime import datetime

BASE_URL = "http://localhost:8080"

async def create_collective_player(session, name, config_type):
    """Create a collective intelligence player"""
    async with session.post(f"{BASE_URL}/api/player/collective/create", json={
        "name": name,
        "config_type": config_type
    }) as resp:
        return await resp.json()

async def create_sota_player(session, model_name, api_key):
    """Create a SOTA single model player"""
    async with session.post(f"{BASE_URL}/api/player/sota/create", json={
        "model_name": model_name,
        "api_key": api_key
    }) as resp:
        return await resp.json()

async def create_game(session, game_type, collective_players, sota_players):
    """Create a new game"""
    async with session.post(f"{BASE_URL}/api/game/create", json={
        "game_type": game_type,
        "rounds": 100,
        "time_limit_ms": 5000,
        "collective_players": collective_players,
        "sota_players": sota_players
    }) as resp:
        return await resp.json()

async def process_turn(session, game_id, actions):
    """Process a game turn"""
    async with session.post(f"{BASE_URL}/api/game/{game_id}/turn", json={
        "actions": actions
    }) as resp:
        return await resp.json()

async def get_game_state(session, game_id):
    """Get current game state"""
    async with session.get(f"{BASE_URL}/api/game/{game_id}/state") as resp:
        return await resp.json()

async def get_analytics(session, game_id):
    """Get game analytics"""
    async with session.get(f"{BASE_URL}/api/analytics/{game_id}") as resp:
        return await resp.json()

async def simulate_minority_game():
    """Run a complete Minority Game simulation"""
    async with aiohttp.ClientSession() as session:
        print("🎮 AI Genius Game Demo - Minority Game")
        print("=" * 50)
        
        # Create players
        print("\n📊 Creating players...")
        
        # Collective players
        opus_orchestra = await create_collective_player(session, "Opus Orchestra", "OpusOrchestra")
        swarm_intel = await create_collective_player(session, "Swarm Intelligence", "SwarmIntelligence")
        
        print(f"✅ Created collective player: {opus_orchestra['player_id']}")
        print(f"✅ Created collective player: {swarm_intel['player_id']}")
        
        # SOTA players
        claude_solo = await create_sota_player(session, "claude-opus-4", "dummy_key")
        gpt4_solo = await create_sota_player(session, "gpt-4-turbo", "dummy_key")
        
        print(f"✅ Created SOTA player: {claude_solo['player_id']}")
        print(f"✅ Created SOTA player: {gpt4_solo['player_id']}")
        
        # Create game
        print("\n🎯 Creating Minority Game...")
        game = await create_game(
            session, 
            "MinorityGame",
            [opus_orchestra['player_id'], swarm_intel['player_id']],
            [claude_solo['player_id'], gpt4_solo['player_id']]
        )
        
        game_id = game['game_id']
        print(f"✅ Game created: {game_id}")
        
        # Simulate 10 rounds
        print("\n🔄 Running game simulation...")
        for round_num in range(10):
            print(f"\n--- Round {round_num + 1} ---")
            
            # Get current state
            state = await get_game_state(session, game_id)
            
            # Generate actions for all players
            actions = {}
            
            # Collective players make decisions
            for player_id in [opus_orchestra['player_id'], swarm_intel['player_id']]:
                actions[player_id] = {
                    "round": round_num,
                    "history": state.get('history', [])
                }
            
            # SOTA players make decisions
            for player_id in [claude_solo['player_id'], gpt4_solo['player_id']]:
                actions[player_id] = {
                    "round": round_num,
                    "history": state.get('history', [])
                }
            
            # Process turn
            result = await process_turn(session, game_id, actions)
            
            if result.get('status') == 'turn_processed':
                round_result = result['round_result']
                print(f"Winners: {round_result['outcome']['winners']}")
                print(f"Emergence detected: {round_result['outcome']['emergence_detected']}")
                
                # Show scores
                current_state = await get_game_state(session, game_id)
                print("\nCurrent scores:")
                for player_id, score in current_state['scores'].items():
                    player_type = "Collective" if player_id.startswith("collective_") else "SOTA"
                    print(f"  {player_type} ({player_id[:12]}...): {score}")
            
            elif result.get('status') == 'game_over':
                print("\n🏁 Game Over!")
                final_result = result['result']
                print(f"Winner: {final_result['winner']}")
                print(f"Total rounds: {final_result['total_rounds']}")
                print(f"Emergence events: {len(final_result['emergence_events'])}")
                break
            
            # Small delay between rounds
            await asyncio.sleep(0.5)
        
        # Get final analytics
        print("\n📈 Final Analytics")
        print("=" * 50)
        analytics = await get_analytics(session, game_id)
        if analytics:
            print(f"Collective emergence frequency: {analytics.get('collective_metrics', {}).get('emergence_frequency', 0):.2%}")
            print(f"Performance differential: {analytics.get('performance_comparison', {}).get('avg_score_differential', 0):.2f}")

async def main():
    """Main demo runner"""
    print("🚀 Starting AI Genius Game Demo")
    print("Make sure the server is running on http://localhost:8080")
    print()
    
    await simulate_minority_game()

if __name__ == "__main__":
    asyncio.run(main())