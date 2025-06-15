"""
Genius Games Python SDK - AI Collective Intelligence Gaming Platform
"""

from .client import GeniusGamesClient
from .types import (
    GameType,
    PlayerInfo,
    PlayerType,
    GameState,
    GameConfig,
    Action,
    RoundResult,
    GameResult,
    ConnectionOptions
)
from .ai_player import AIPlayer, AIDecisionMaker, RandomDecisionMaker, StrategicDecisionMaker
from .swarm import AISwarm, create_ai_swarm
from .simulator import GameSimulator, SimulationResults

__version__ = "1.0.0"
__all__ = [
    # Client
    "GeniusGamesClient",
    
    # Types
    "GameType",
    "PlayerInfo", 
    "PlayerType",
    "GameState",
    "GameConfig",
    "Action",
    "RoundResult",
    "GameResult",
    "ConnectionOptions",
    
    # AI
    "AIPlayer",
    "AIDecisionMaker",
    "RandomDecisionMaker",
    "StrategicDecisionMaker",
    
    # Swarm
    "AISwarm",
    "create_ai_swarm",
    
    # Simulator
    "GameSimulator",
    "SimulationResults"
]