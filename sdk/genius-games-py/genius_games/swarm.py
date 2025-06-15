"""
AI Swarm management for Genius Games
"""

import asyncio
from typing import List, Dict, Any, Optional, Union
import logging
from dataclasses import dataclass

from .ai_player import AIPlayer, AIDecisionMaker, RandomDecisionMaker, StrategicDecisionMaker
from .types import PlayerInfo, PlayerType, GameConfig, GameResult, GameType


@dataclass
class SwarmStats:
    """Statistics about the swarm"""
    total_players: int
    active_players: int
    games_played: int
    total_wins: Dict[str, int]
    win_rates: Dict[str, float]
    
    def update_win_rates(self):
        """Update win rates based on total wins"""
        if self.games_played > 0:
            for player, wins in self.total_wins.items():
                self.win_rates[player] = wins / self.games_played


class AISwarm:
    """Manages a swarm of AI players"""
    
    def __init__(self, players: List[AIPlayer]):
        """Initialize swarm with AI players"""
        self.players = players
        self.logger = logging.getLogger("AISwarm")
        self.stats = SwarmStats(
            total_players=len(players),
            active_players=0,
            games_played=0,
            total_wins={p.player_info.name: 0 for p in players},
            win_rates={}
        )
        self._active = False
    
    async def connect_all(self) -> None:
        """Connect all players to the server"""
        self.logger.info(f"Connecting {len(self.players)} players...")
        
        tasks = [player.connect() for player in self.players]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        connected = sum(1 for r in results if not isinstance(r, Exception))
        self.stats.active_players = connected
        
        if connected < len(self.players):
            self.logger.warning(f"Only {connected}/{len(self.players)} players connected")
        else:
            self.logger.info("All players connected successfully")
        
        self._active = True
    
    async def disconnect_all(self) -> None:
        """Disconnect all players"""
        self.logger.info("Disconnecting all players...")
        
        tasks = [player.disconnect() for player in self.players]
        await asyncio.gather(*tasks, return_exceptions=True)
        
        self.stats.active_players = 0
        self._active = False
        self.logger.info("All players disconnected")
    
    def disconnect(self) -> None:
        """Synchronous disconnect for convenience"""
        if self._active:
            asyncio.create_task(self.disconnect_all())
    
    async def create_and_play_game(self, game_config: Union[GameConfig, Dict[str, Any]]) -> List[GameResult]:
        """Have one player create a game and all players join"""
        if not self._active or self.stats.active_players == 0:
            raise Exception("Swarm not connected")
        
        # Extract game type
        if isinstance(game_config, dict):
            game_type_str = game_config.get("gameType", "")
            game_type = GameType(game_type_str) if game_type_str else GameType.MINORITY_GAME
        else:
            game_type = game_config.game_type if isinstance(game_config.game_type, GameType) else GameType(game_config.game_type)
        
        # First player creates the game
        creator = self.players[0]
        self.logger.info(f"Player {creator.player_info.name} creating game...")
        
        # Create game
        game_id = await creator.client.create_game(game_config)
        creator.current_game_type = game_type
        
        # Other players join
        join_tasks = []
        for player in self.players[1:]:
            player.current_game_type = game_type
            join_tasks.append(player.client.join_game(game_id))
        
        # Wait for all to join
        join_results = await asyncio.gather(*join_tasks, return_exceptions=True)
        joined = sum(1 for r in join_results if not isinstance(r, Exception)) + 1  # +1 for creator
        
        self.logger.info(f"Game {game_id} started with {joined} players")
        
        # Play the game
        play_tasks = []
        
        # Creator already in game, just needs to wait for end
        creator_future = asyncio.get_event_loop().create_future()
        creator._game_result_future = creator_future
        play_tasks.append(creator_future)
        
        # Other players need to wait for game end
        for i, player in enumerate(self.players[1:]):
            if not isinstance(join_results[i], Exception):
                player_future = asyncio.get_event_loop().create_future()
                player._game_result_future = player_future
                play_tasks.append(player_future)
        
        # Wait for game to complete
        results = await asyncio.gather(*play_tasks, return_exceptions=True)
        
        # Process results
        game_results = [r for r in results if isinstance(r, GameResult)]
        
        if game_results:
            # Update statistics
            self.stats.games_played += 1
            game_result = game_results[0]  # All should be the same
            
            # Update wins
            if game_result.winners:
                for winner in game_result.winners:
                    if winner in self.stats.total_wins:
                        self.stats.total_wins[winner] += 1
            elif game_result.winner and game_result.winner in self.stats.total_wins:
                self.stats.total_wins[game_result.winner] += 1
            
            self.stats.update_win_rates()
            
            self.logger.info(f"Game completed. Winners: {game_result.winners or [game_result.winner]}")
        
        return game_results
    
    async def run_tournament(self, game_configs: List[Union[GameConfig, Dict[str, Any]]]) -> Dict[str, Any]:
        """Run a tournament with multiple games"""
        self.logger.info(f"Starting tournament with {len(game_configs)} games")
        
        all_results = []
        
        for i, config in enumerate(game_configs):
            self.logger.info(f"\nGame {i+1}/{len(game_configs)}")
            
            try:
                results = await self.create_and_play_game(config)
                all_results.extend(results)
                
                # Brief pause between games
                await asyncio.sleep(2)
                
            except Exception as e:
                self.logger.error(f"Game {i+1} failed: {e}")
        
        # Compile tournament results
        tournament_stats = {
            "total_games": len(game_configs),
            "completed_games": self.stats.games_played,
            "player_stats": {
                name: {
                    "wins": wins,
                    "win_rate": self.stats.win_rates.get(name, 0),
                }
                for name, wins in self.stats.total_wins.items()
            },
            "swarm_size": self.stats.total_players,
            "active_players": self.stats.active_players
        }
        
        self.logger.info("\n=== TOURNAMENT COMPLETE ===")
        self.logger.info(f"Games played: {tournament_stats['completed_games']}/{tournament_stats['total_games']}")
        self.logger.info("\nTop performers:")
        
        sorted_players = sorted(
            tournament_stats["player_stats"].items(),
            key=lambda x: x[1]["win_rate"],
            reverse=True
        )[:5]
        
        for name, stats in sorted_players:
            self.logger.info(f"  {name}: {stats['wins']} wins ({stats['win_rate']*100:.1f}%)")
        
        return tournament_stats
    
    def get_stats(self) -> Dict[str, Any]:
        """Get current swarm statistics"""
        return {
            "totalPlayers": self.stats.total_players,
            "activePlayers": self.stats.active_players,
            "gamesPlayed": self.stats.games_played,
            "winRates": self.stats.win_rates,
            "totalWins": self.stats.total_wins
        }


async def create_ai_swarm(
    url: str,
    size: int,
    name_prefix: str = "AI",
    decision_maker: Union[str, AIDecisionMaker] = "random",
    connect: bool = True,
    **connection_options
) -> AISwarm:
    """Create a swarm of AI players"""
    
    players = []
    
    for i in range(size):
        # Create player info
        player_info = PlayerInfo(
            id=f"{name_prefix.lower()}-{i+1}",
            name=f"{name_prefix}-{i+1}",
            type=PlayerType.AI
        )
        
        # Create decision maker
        if isinstance(decision_maker, str):
            if decision_maker == "random":
                dm = RandomDecisionMaker(name=player_info.name)
            elif decision_maker == "strategic":
                dm = StrategicDecisionMaker(name=player_info.name)
            elif decision_maker == "mixed":
                # Mix of random and strategic
                dm = RandomDecisionMaker(name=player_info.name) if i % 2 == 0 else StrategicDecisionMaker(name=player_info.name)
            else:
                dm = RandomDecisionMaker(name=player_info.name)
        else:
            dm = decision_maker
        
        # Create AI player
        player = AIPlayer(url, player_info, dm, **connection_options)
        players.append(player)
    
    # Create swarm
    swarm = AISwarm(players)
    
    # Connect if requested
    if connect:
        await swarm.connect_all()
    
    return swarm