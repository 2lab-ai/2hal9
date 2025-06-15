"""
Type definitions for Genius Games SDK
"""

from typing import Dict, List, Any, Optional, Union, Literal
from enum import Enum
from dataclasses import dataclass, field
from datetime import datetime


class GameType(str, Enum):
    """Supported game types"""
    MINORITY_GAME = "minority_game"
    BYZANTINE_GENERALS = "byzantine_generals"
    PRISONERS_DILEMMA = "prisoners_dilemma"
    QUANTUM_CONSENSUS = "quantum_consensus"
    COLLECTIVE_MAZE = "collective_maze"
    RECURSIVE_REASONING = "recursive_reasoning"
    SWARM_OPTIMIZATION = "swarm_optimization"
    MINI_GO = "mini_go"
    MINI_HOLDEM = "mini_holdem"
    # Death games
    SQUID_GAME = "squid_game"
    BATTLE_ROYALE = "battle_royale"
    HUNGER_GAMES = "hunger_games"
    LIARS_DICE = "liars_dice"
    RUSSIAN_ROULETTE = "russian_roulette"
    KING_OF_THE_HILL = "king_of_the_hill"
    LAST_STAND = "last_stand"
    TRUST_FALL = "trust_fall"


class PlayerType(str, Enum):
    """Player types"""
    HUMAN = "human"
    AI = "ai"
    HYBRID = "hybrid"


@dataclass
class PlayerInfo:
    """Player information"""
    id: str
    name: str
    type: Union[PlayerType, str] = PlayerType.HUMAN
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ConnectionOptions:
    """WebSocket connection options"""
    url: str
    player_info: Optional[PlayerInfo] = None
    reconnect: bool = True
    reconnect_interval: int = 5000  # ms
    max_reconnects: int = 5
    timeout: int = 30000  # ms
    debug: bool = False
    headers: Dict[str, str] = field(default_factory=dict)


@dataclass
class Action:
    """Player action"""
    action_type: str
    data: Dict[str, Any] = field(default_factory=dict)
    reasoning: Optional[str] = None
    confidence: float = 1.0
    timestamp: Optional[datetime] = None


@dataclass
class GameConfig:
    """Game configuration"""
    game_type: Union[GameType, str]
    rounds: int = 10
    time_limit_ms: Optional[int] = None
    players: Optional[List[PlayerInfo]] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    

@dataclass
class Position:
    """2D position"""
    x: int
    y: int


@dataclass
class PlayerState:
    """Individual player state within a game"""
    id: str
    name: str
    score: int = 0
    alive: bool = True
    position: Optional[Position] = None
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class GameState:
    """Current game state"""
    game_id: str
    game_type: Union[GameType, str]
    round: int
    max_rounds: int
    players: List[PlayerState]
    scores: Dict[str, int]
    alive_players: List[str]
    current_player: Optional[str] = None
    phase: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    @property
    def is_final_round(self) -> bool:
        """Check if this is the final round"""
        return self.round >= self.max_rounds
    
    def get_player_state(self, player_id: str) -> Optional[PlayerState]:
        """Get state for specific player"""
        for player in self.players:
            if player.id == player_id:
                return player
        return None


@dataclass 
class RoundOutcome:
    """Outcome of a single round"""
    winners: List[str]
    losers: List[str] 
    eliminated: List[str] = field(default_factory=list)
    scores: Dict[str, int] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class RoundResult:
    """Result of a game round"""
    game_id: str
    round: int
    actions: Dict[str, Action]
    outcome: RoundOutcome
    state: GameState
    timestamp: datetime = field(default_factory=datetime.now)


@dataclass
class GameResult:
    """Final game result"""
    game_id: str
    game_type: Union[GameType, str]
    winner: Optional[str]
    winners: List[str]
    final_scores: Dict[str, int]
    rounds_played: int
    start_time: datetime
    end_time: datetime
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    @property
    def duration_seconds(self) -> float:
        """Get game duration in seconds"""
        return (self.end_time - self.start_time).total_seconds()


@dataclass
class SimulationResults:
    """Results from running simulations"""
    total_games: int
    completed_games: int
    failed_games: int
    win_rates: Dict[str, float]
    avg_scores: Dict[str, float]
    avg_game_duration: float
    emergence_events: int
    metadata: Dict[str, Any] = field(default_factory=dict)


# WebSocket message types
class MessageType(str, Enum):
    """WebSocket message types"""
    # Client to server
    JOIN_GAME = "join_game"
    CREATE_GAME = "create_game"
    SUBMIT_ACTION = "submit_action"
    LEAVE_GAME = "leave_game"
    
    # Server to client
    GAME_CREATED = "game_created"
    GAME_JOINED = "game_joined"
    GAME_STATE_UPDATE = "game_state_update"
    ROUND_RESULT = "round_result"
    GAME_ENDED = "game_ended"
    ERROR = "error"
    
    # Connection
    PING = "ping"
    PONG = "pong"