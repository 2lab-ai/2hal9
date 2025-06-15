"""
Game simulator for running experiments and analysis
"""

import asyncio
import time
from typing import List, Dict, Any, Optional, Union
import logging
import statistics
from datetime import datetime

from .swarm import create_ai_swarm, AISwarm
from .types import GameConfig, GameType, SimulationResults


class GameSimulator:
    """Simulator for running game experiments"""
    
    def __init__(self, server_url: str, logger: Optional[logging.Logger] = None):
        """Initialize simulator"""
        self.server_url = server_url
        self.logger = logger or logging.getLogger("GameSimulator")
    
    async def run_simulation(
        self,
        games: List[Union[GameConfig, Dict[str, Any]]],
        swarm_size: int,
        swarm_type: str = "mixed",
        rounds: int = 10,
        verbose: bool = True
    ) -> SimulationResults:
        """Run a simulation with multiple games and rounds"""
        
        start_time = time.time()
        self.logger.info(f"Starting simulation: {len(games)} game types, {rounds} rounds each, {swarm_size} players")
        
        # Create results tracking
        all_game_results = []
        total_games = len(games) * rounds
        completed_games = 0
        failed_games = 0
        player_wins: Dict[str, int] = {}
        player_scores: Dict[str, List[int]] = {}
        game_durations: List[float] = []
        emergence_events = 0
        
        # Create AI swarm
        swarm = await create_ai_swarm(
            self.server_url,
            size=swarm_size,
            name_prefix=f"Sim{swarm_type.capitalize()}",
            decision_maker=swarm_type,
            debug=False
        )
        
        try:
            # Run simulations
            for game_config in games:
                # Extract game type for logging
                if isinstance(game_config, dict):
                    game_type = game_config.get("gameType", "unknown")
                else:
                    game_type = game_config.game_type
                
                self.logger.info(f"\nSimulating {game_type} - {rounds} rounds")
                
                for round_num in range(rounds):
                    if verbose:
                        self.logger.info(f"  Round {round_num + 1}/{rounds}")
                    
                    round_start = time.time()
                    
                    try:
                        # Run game
                        results = await swarm.create_and_play_game(game_config)
                        
                        if results:
                            completed_games += 1
                            result = results[0]  # All players get same result
                            
                            # Track game duration
                            game_durations.append(time.time() - round_start)
                            
                            # Update win tracking
                            if result.winners:
                                for winner in result.winners:
                                    player_wins[winner] = player_wins.get(winner, 0) + 1
                            elif result.winner:
                                player_wins[result.winner] = player_wins.get(result.winner, 0) + 1
                            
                            # Update score tracking
                            for player_id, score in result.final_scores.items():
                                if player_id not in player_scores:
                                    player_scores[player_id] = []
                                player_scores[player_id].append(score)
                            
                            # Check for emergence events
                            if self._check_emergence(result, swarm_size):
                                emergence_events += 1
                            
                            all_game_results.append(result)
                        else:
                            failed_games += 1
                    
                    except Exception as e:
                        self.logger.error(f"Game failed: {e}")
                        failed_games += 1
                    
                    # Brief pause between games
                    await asyncio.sleep(1)
            
            # Calculate final statistics
            win_rates = {}
            avg_scores = {}
            
            for player_id in player_scores:
                # Win rate
                wins = player_wins.get(player_id, 0)
                win_rates[player_id] = wins / completed_games if completed_games > 0 else 0
                
                # Average score
                scores = player_scores[player_id]
                avg_scores[player_id] = statistics.mean(scores) if scores else 0
            
            # Average game duration
            avg_duration = statistics.mean(game_durations) if game_durations else 0
            
            # Create results
            results = SimulationResults(
                total_games=total_games,
                completed_games=completed_games,
                failed_games=failed_games,
                win_rates=win_rates,
                avg_scores=avg_scores,
                avg_game_duration=avg_duration,
                emergence_events=emergence_events,
                metadata={
                    "swarm_size": swarm_size,
                    "swarm_type": swarm_type,
                    "duration_seconds": time.time() - start_time,
                    "games_per_second": completed_games / (time.time() - start_time) if completed_games > 0 else 0
                }
            )
            
            # Log summary
            self.logger.info("\n=== SIMULATION COMPLETE ===")
            self.logger.info(f"Total games: {total_games}")
            self.logger.info(f"Completed: {completed_games} ({completed_games/total_games*100:.1f}%)")
            self.logger.info(f"Failed: {failed_games}")
            self.logger.info(f"Average game duration: {avg_duration:.2f}s")
            self.logger.info(f"Emergence events: {emergence_events}")
            self.logger.info(f"Total time: {time.time() - start_time:.1f}s")
            
            if verbose:
                # Top performers
                self.logger.info("\nTop 5 by win rate:")
                top_winners = sorted(win_rates.items(), key=lambda x: x[1], reverse=True)[:5]
                for player, rate in top_winners:
                    self.logger.info(f"  {player}: {rate*100:.1f}%")
                
                self.logger.info("\nTop 5 by average score:")
                top_scorers = sorted(avg_scores.items(), key=lambda x: x[1], reverse=True)[:5]
                for player, score in top_scorers:
                    self.logger.info(f"  {player}: {score:.1f}")
            
            return results
            
        finally:
            # Disconnect swarm
            await swarm.disconnect_all()
    
    def _check_emergence(self, game_result: Any, swarm_size: int) -> bool:
        """Check if emergence behavior occurred"""
        # Simple heuristic: unusual concentration of winners
        if hasattr(game_result, 'winners') and game_result.winners:
            winner_ratio = len(game_result.winners) / swarm_size
            # Emergence if very few or very many winners (unusual consensus)
            return winner_ratio < 0.1 or winner_ratio > 0.9
        return False
    
    async def quick_test(self, game_type: GameType, players: int = 4) -> Dict[str, Any]:
        """Run a quick test of a single game"""
        self.logger.info(f"Quick test: {game_type} with {players} players")
        
        # Create small swarm
        swarm = await create_ai_swarm(
            self.server_url,
            size=players,
            name_prefix="QuickTest",
            decision_maker="mixed"
        )
        
        try:
            # Run one game
            config = {
                "gameType": game_type.value,
                "rounds": 10,
                "timeLimitMs": 5000
            }
            
            start = time.time()
            results = await swarm.create_and_play_game(config)
            duration = time.time() - start
            
            if results:
                result = results[0]
                return {
                    "success": True,
                    "duration": duration,
                    "winner": result.winner,
                    "winners": result.winners,
                    "final_scores": result.final_scores,
                    "rounds_played": result.rounds_played
                }
            else:
                return {
                    "success": False,
                    "error": "No results returned"
                }
                
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
        finally:
            await swarm.disconnect_all()
    
    async def benchmark_games(self, players: int = 8) -> Dict[str, Any]:
        """Benchmark all game types"""
        self.logger.info(f"Benchmarking all games with {players} players")
        
        benchmark_results = {}
        
        # Test each game type
        for game_type in GameType:
            self.logger.info(f"\nBenchmarking {game_type.value}...")
            
            try:
                result = await self.quick_test(game_type, players)
                benchmark_results[game_type.value] = result
                
                if result["success"]:
                    self.logger.info(f"✓ {game_type.value}: {result['duration']:.2f}s")
                else:
                    self.logger.info(f"✗ {game_type.value}: {result['error']}")
                
            except Exception as e:
                self.logger.error(f"✗ {game_type.value}: {str(e)}")
                benchmark_results[game_type.value] = {
                    "success": False,
                    "error": str(e)
                }
            
            # Brief pause
            await asyncio.sleep(1)
        
        # Summary
        successful = sum(1 for r in benchmark_results.values() if r.get("success", False))
        self.logger.info(f"\n=== BENCHMARK COMPLETE ===")
        self.logger.info(f"Successful: {successful}/{len(GameType)}")
        
        return benchmark_results