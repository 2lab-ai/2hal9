#!/usr/bin/env python3
"""
AI Swarm example for Genius Games Python SDK
"""

import asyncio
import logging
from genius_games import (
    create_ai_swarm,
    GameType,
    GameSimulator,
    AIPlayer,
    PlayerInfo,
    PlayerType,
    StrategicDecisionMaker
)

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)


async def simple_swarm_demo():
    """Demo with a simple AI swarm"""
    print("🤖 Simple AI Swarm Demo\n")
    
    # Create a swarm of 8 AI players
    print("Creating swarm of 8 AI players...")
    swarm = await create_ai_swarm(
        url="http://localhost:8080",
        size=8,
        name_prefix="SwarmBot",
        decision_maker="strategic",  # Use strategic AI
        debug=False
    )
    
    print("✅ Swarm connected!\n")
    
    try:
        # Play different games
        games = [
            {
                "name": "Minority Game",
                "config": {
                    "gameType": GameType.MINORITY_GAME.value,
                    "rounds": 20,
                    "timeLimitMs": 5000
                }
            },
            {
                "name": "Prisoner's Dilemma",
                "config": {
                    "gameType": GameType.PRISONERS_DILEMMA.value,
                    "rounds": 15,
                    "timeLimitMs": 5000
                }
            },
            {
                "name": "Battle Royale",
                "config": {
                    "gameType": GameType.BATTLE_ROYALE.value,
                    "rounds": 50,
                    "timeLimitMs": 3000
                }
            }
        ]
        
        for game in games:
            print(f"🎮 Starting {game['name']}...")
            
            results = await swarm.create_and_play_game(game['config'])
            
            if results:
                result = results[0]
                print(f"✅ Game completed!")
                print(f"   Winners: {result.winners or [result.winner]}")
                print(f"   Duration: {result.duration_seconds:.1f}s")
                print(f"   Rounds: {result.rounds_played}")
                
                # Show top scores
                top_scores = sorted(
                    result.final_scores.items(),
                    key=lambda x: x[1],
                    reverse=True
                )[:3]
                print("   Top 3 scores:")
                for player, score in top_scores:
                    print(f"     {player}: {score}")
                print()
            
            # Brief pause between games
            await asyncio.sleep(2)
        
        # Show swarm statistics
        stats = swarm.get_stats()
        print("\n📊 Swarm Statistics:")
        print(f"   Total players: {stats['totalPlayers']}")
        print(f"   Games played: {stats['gamesPlayed']}")
        print("\n   Win rates:")
        
        sorted_rates = sorted(
            stats['winRates'].items(),
            key=lambda x: x[1],
            reverse=True
        )
        for player, rate in sorted_rates[:5]:
            print(f"     {player}: {rate*100:.1f}%")
    
    finally:
        print("\n👋 Disconnecting swarm...")
        await swarm.disconnect_all()
        print("✅ Demo complete!")


async def large_swarm_demo():
    """Demo with a large swarm (16 players)"""
    print("🤖 Large Swarm Demo - Testing Collective Intelligence\n")
    
    # Create large swarm
    print("Creating swarm of 16 AI players...")
    swarm = await create_ai_swarm(
        url="http://localhost:8080",
        size=16,
        name_prefix="Collective",
        decision_maker="mixed",  # Mix of random and strategic
        debug=False
    )
    
    print("✅ Large swarm connected!\n")
    
    try:
        # Test emergence in different games
        emergence_games = [
            {
                "name": "Byzantine Generals",
                "config": {
                    "gameType": GameType.BYZANTINE_GENERALS.value,
                    "rounds": 20,
                    "timeLimitMs": 5000
                },
                "description": "Testing consensus with potential traitors"
            },
            {
                "name": "Quantum Consensus",
                "config": {
                    "gameType": GameType.QUANTUM_CONSENSUS.value,
                    "rounds": 15,
                    "timeLimitMs": 5000
                },
                "description": "Testing quantum-inspired collective decision making"
            },
            {
                "name": "Trust Fall",
                "config": {
                    "gameType": GameType.TRUST_FALL.value,
                    "rounds": 25,
                    "timeLimitMs": 3000
                },
                "description": "Testing trust dynamics in large groups"
            }
        ]
        
        for game in emergence_games:
            print(f"🎮 {game['name']}: {game['description']}")
            
            results = await swarm.create_and_play_game(game['config'])
            
            if results:
                result = results[0]
                
                # Analyze emergence patterns
                winner_count = len(result.winners) if result.winners else 1
                consensus_ratio = winner_count / 16
                
                print(f"✅ Completed!")
                print(f"   Winners: {winner_count}/16 players ({consensus_ratio*100:.1f}%)")
                print(f"   Emergence indicator: ", end="")
                
                if consensus_ratio > 0.8:
                    print("Strong consensus achieved! 🌟")
                elif consensus_ratio > 0.6:
                    print("Moderate consensus")
                elif consensus_ratio < 0.2:
                    print("High competition/fragmentation")
                else:
                    print("Mixed outcomes")
                
                print()
            
            await asyncio.sleep(2)
    
    finally:
        await swarm.disconnect_all()


async def simulation_demo():
    """Demo using the game simulator"""
    print("📊 Game Simulation Demo\n")
    
    simulator = GameSimulator("http://localhost:8080")
    
    # Define simulation parameters
    games = [
        {
            "gameType": GameType.MINORITY_GAME.value,
            "rounds": 30,
            "timeLimitMs": 3000
        },
        {
            "gameType": GameType.PRISONERS_DILEMMA.value,
            "rounds": 20,
            "timeLimitMs": 3000
        },
        {
            "gameType": GameType.BATTLE_ROYALE.value,
            "rounds": 50,
            "timeLimitMs": 2000
        }
    ]
    
    print("Running simulation with:")
    print("  - 3 different game types")
    print("  - 10 rounds per game")
    print("  - 8 AI players (mixed strategies)")
    print()
    
    # Run simulation
    results = await simulator.run_simulation(
        games=games,
        swarm_size=8,
        swarm_type="mixed",
        rounds=10,
        verbose=True
    )
    
    print("\n=== FINAL RESULTS ===")
    print(f"Success rate: {results.completed_games}/{results.total_games} ({results.completed_games/results.total_games*100:.1f}%)")
    print(f"Average game duration: {results.avg_game_duration:.2f}s")
    print(f"Emergence events detected: {results.emergence_events}")
    
    if results.metadata:
        print(f"Games per second: {results.metadata['games_per_second']:.2f}")
        print(f"Total simulation time: {results.metadata['duration_seconds']:.1f}s")


async def custom_ai_demo():
    """Demo with custom AI implementation"""
    print("🤖 Custom AI Player Demo\n")
    
    # Create custom AI player
    from genius_games import AIDecisionMaker
    
    class AggressiveAI(AIDecisionMaker):
        """Always chooses aggressive/competitive options"""
        
        async def make_decision(self, game_state, game_type):
            if game_type == GameType.PRISONERS_DILEMMA:
                return Action(
                    action_type="defect",
                    reasoning="Always defect strategy",
                    confidence=1.0
                )
            elif game_type == GameType.MINORITY_GAME:
                # Try to be contrarian
                if game_state.round % 3 == 0:
                    choice = "choose_1"
                else:
                    choice = "choose_0"
                return Action(
                    action_type=choice,
                    reasoning="Aggressive contrarian",
                    confidence=0.9
                )
            else:
                return Action(
                    action_type="default",
                    reasoning="Aggressive default",
                    confidence=0.8
                )
        
        def get_name(self):
            return "AggressiveAI"
    
    # Create player with custom AI
    player = AIPlayer(
        url="http://localhost:8080",
        player_info=PlayerInfo(
            id="aggressive-1",
            name="AggressiveBot",
            type=PlayerType.AI
        ),
        decision_maker=AggressiveAI()
    )
    
    try:
        await player.connect()
        print("✅ Custom AI connected!\n")
        
        # Play a game
        print("Creating Prisoner's Dilemma game...")
        result = await player.create_and_play({
            "gameType": GameType.PRISONERS_DILEMMA.value,
            "rounds": 10,
            "timeLimitMs": 5000
        })
        
        print(f"\n✅ Game completed!")
        print(f"Winner: {result.winner}")
        print(f"AggressiveBot score: {result.final_scores.get('AggressiveBot', 0)}")
        
    finally:
        await player.disconnect()


async def main():
    """Main menu for demos"""
    print("🎮 Genius Games Python SDK - AI Swarm Examples\n")
    print("Choose a demo:")
    print("1. Simple swarm (8 players)")
    print("2. Large swarm emergence (16 players)")
    print("3. Game simulation suite")
    print("4. Custom AI implementation")
    print("5. Run all demos")
    
    choice = input("\nYour choice (1-5): ").strip()
    
    demos = {
        "1": simple_swarm_demo,
        "2": large_swarm_demo,
        "3": simulation_demo,
        "4": custom_ai_demo
    }
    
    if choice == "5":
        # Run all demos
        for name, demo in demos.items():
            print(f"\n{'='*60}")
            print(f"Running demo {name}...")
            print(f"{'='*60}\n")
            await demo()
            await asyncio.sleep(2)
    elif choice in demos:
        await demos[choice]()
    else:
        print("Invalid choice. Running simple swarm demo...")
        await simple_swarm_demo()


if __name__ == "__main__":
    asyncio.run(main())