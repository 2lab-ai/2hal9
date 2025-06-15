"""
AI player implementation for Genius Games
"""

import asyncio
import random
from abc import ABC, abstractmethod
from typing import Optional, Dict, Any, List
import logging

from .client import GeniusGamesClient
from .types import (
    PlayerInfo,
    PlayerType,
    GameState,
    GameType,
    Action,
    GameConfig,
    RoundResult,
    GameResult
)


class AIDecisionMaker(ABC):
    """Abstract base class for AI decision making"""
    
    @abstractmethod
    async def make_decision(self, game_state: GameState, game_type: GameType) -> Action:
        """Make a decision based on the current game state"""
        pass
    
    @abstractmethod
    def get_name(self) -> str:
        """Get the name of this AI"""
        pass


class RandomDecisionMaker(AIDecisionMaker):
    """Random decision maker for baseline AI"""
    
    def __init__(self, name: str = "RandomAI"):
        self.name = name
    
    async def make_decision(self, game_state: GameState, game_type: GameType) -> Action:
        """Make a random decision"""
        # Game-specific random actions
        if game_type == GameType.MINORITY_GAME:
            action_type = random.choice(["choose_0", "choose_1"])
            reasoning = "Random choice"
        
        elif game_type == GameType.PRISONERS_DILEMMA:
            action_type = random.choice(["cooperate", "defect"])
            reasoning = "Random cooperation/defection"
        
        elif game_type == GameType.BYZANTINE_GENERALS:
            action_type = random.choice(["attack", "retreat"])
            reasoning = "Random military decision"
        
        elif game_type == GameType.BATTLE_ROYALE:
            directions = ["north", "south", "east", "west", "stay"]
            action_type = "move"
            data = {"direction": random.choice(directions)}
            reasoning = "Random movement"
            return Action(
                action_type=action_type,
                data=data,
                reasoning=reasoning,
                confidence=0.5
            )
        
        elif game_type == GameType.MINI_GO:
            # Random valid move (simplified)
            x = random.randint(0, 8)
            y = random.randint(0, 8)
            action_type = "place_stone"
            data = {"x": x, "y": y}
            reasoning = f"Random placement at ({x}, {y})"
            return Action(
                action_type=action_type,
                data=data,
                reasoning=reasoning,
                confidence=0.3
            )
        
        else:
            # Generic random action
            action_type = "default"
            reasoning = "No specific strategy"
        
        return Action(
            action_type=action_type,
            data={},
            reasoning=reasoning,
            confidence=0.5
        )
    
    def get_name(self) -> str:
        return self.name


class StrategicDecisionMaker(AIDecisionMaker):
    """Strategic AI that analyzes game state"""
    
    def __init__(self, name: str = "StrategicAI"):
        self.name = name
        self.history: List[RoundResult] = []
        self.opponent_patterns: Dict[str, Dict[str, int]] = {}
    
    async def make_decision(self, game_state: GameState, game_type: GameType) -> Action:
        """Make a strategic decision based on analysis"""
        
        if game_type == GameType.MINORITY_GAME:
            return await self._decide_minority_game(game_state)
        
        elif game_type == GameType.PRISONERS_DILEMMA:
            return await self._decide_prisoners_dilemma(game_state)
        
        elif game_type == GameType.BYZANTINE_GENERALS:
            return await self._decide_byzantine_generals(game_state)
        
        elif game_type == GameType.BATTLE_ROYALE:
            return await self._decide_battle_royale(game_state)
        
        elif game_type == GameType.TRUST_FALL:
            return await self._decide_trust_fall(game_state)
        
        else:
            # Fallback to random
            return await RandomDecisionMaker().make_decision(game_state, game_type)
    
    async def _decide_minority_game(self, game_state: GameState) -> Action:
        """Strategic decision for Minority Game"""
        # Analyze recent patterns
        if len(self.history) > 3:
            # Look for patterns in majority choices
            recent_majorities = []
            for result in self.history[-3:]:
                if "majority_choice" in result.outcome.metadata:
                    recent_majorities.append(result.outcome.metadata["majority_choice"])
            
            # Anti-pattern strategy
            if len(recent_majorities) >= 2 and recent_majorities[-1] == recent_majorities[-2]:
                # Pattern detected, go opposite
                choice = "choose_1" if recent_majorities[-1] == "0" else "choose_0"
                reasoning = "Detected pattern, going opposite"
                confidence = 0.8
            else:
                # No clear pattern, use probability
                choice = "choose_0" if game_state.round % 3 == 0 else "choose_1"
                reasoning = "Probabilistic anti-majority"
                confidence = 0.6
        else:
            # Not enough history
            choice = "choose_0" if random.random() > 0.5 else "choose_1"
            reasoning = "Initial exploration"
            confidence = 0.5
        
        return Action(
            action_type=choice,
            data={},
            reasoning=reasoning,
            confidence=confidence
        )
    
    async def _decide_prisoners_dilemma(self, game_state: GameState) -> Action:
        """Strategic decision for Prisoner's Dilemma"""
        # Tit-for-tat with forgiveness
        my_id = self.name
        
        # Check opponent's last action
        if len(self.history) > 0:
            last_round = self.history[-1]
            opponent_defected = False
            
            for player_id, action in last_round.actions.items():
                if player_id != my_id and action.action_type == "defect":
                    opponent_defected = True
                    break
            
            if opponent_defected:
                # Retaliate but with 10% forgiveness
                if random.random() < 0.1:
                    action_type = "cooperate"
                    reasoning = "Forgiving defection"
                else:
                    action_type = "defect"
                    reasoning = "Retaliating defection"
            else:
                # Cooperate if opponent cooperated
                action_type = "cooperate"
                reasoning = "Reciprocating cooperation"
        else:
            # First round: cooperate
            action_type = "cooperate"
            reasoning = "Initial cooperation"
        
        confidence = 0.8 if len(self.history) > 2 else 0.6
        
        return Action(
            action_type=action_type,
            data={},
            reasoning=reasoning,
            confidence=confidence
        )
    
    async def _decide_byzantine_generals(self, game_state: GameState) -> Action:
        """Strategic decision for Byzantine Generals"""
        # Analyze consensus likelihood
        alive_count = len(game_state.alive_players)
        traitor_likelihood = min(0.3, 1.0 / alive_count)
        
        # If many players alive, higher chance of consensus
        if alive_count > 5:
            action_type = "attack" if random.random() > 0.3 else "retreat"
            reasoning = f"Large group ({alive_count} players), favoring attack"
            confidence = 0.7
        else:
            # Small group, be more cautious
            action_type = "retreat" if random.random() > 0.6 else "attack"
            reasoning = f"Small group ({alive_count} players), being cautious"
            confidence = 0.6
        
        return Action(
            action_type=action_type,
            data={},
            reasoning=reasoning,
            confidence=confidence
        )
    
    async def _decide_battle_royale(self, game_state: GameState) -> Action:
        """Strategic decision for Battle Royale"""
        my_state = game_state.get_player_state(self.name)
        if not my_state or not my_state.position:
            # No position info, move randomly
            return Action(
                action_type="move",
                data={"direction": random.choice(["north", "south", "east", "west"])},
                reasoning="No position data",
                confidence=0.3
            )
        
        # Strategy: Move toward center early, away from center late
        center_x, center_y = 10, 10  # Assuming 20x20 map
        my_x, my_y = my_state.position.x, my_state.position.y
        
        if game_state.round < game_state.max_rounds * 0.6:
            # Early game: move toward center
            if my_x < center_x:
                direction = "east"
            elif my_x > center_x:
                direction = "west"
            elif my_y < center_y:
                direction = "south"
            else:
                direction = "north"
            reasoning = "Moving toward center (early game)"
        else:
            # Late game: survival mode
            if len(game_state.alive_players) > 3:
                # Still many players, find edges
                if my_x < 5:
                    direction = "west"
                elif my_x > 15:
                    direction = "east"
                elif my_y < 5:
                    direction = "north"
                else:
                    direction = "south"
                reasoning = "Moving to edges (late game survival)"
            else:
                # Final players, stay put or minimal movement
                direction = "stay" if random.random() > 0.3 else random.choice(["north", "south"])
                reasoning = "Final standoff positioning"
        
        return Action(
            action_type="move",
            data={"direction": direction},
            reasoning=reasoning,
            confidence=0.7
        )
    
    async def _decide_trust_fall(self, game_state: GameState) -> Action:
        """Strategic decision for Trust Fall"""
        # Analyze trust levels based on history
        if len(self.history) < 2:
            # Early game: build trust
            action_type = "catch"
            reasoning = "Building initial trust"
            confidence = 0.7
        else:
            # Check recent betrayals
            recent_betrayals = 0
            for result in self.history[-3:]:
                if "betrayals" in result.outcome.metadata:
                    recent_betrayals += result.outcome.metadata["betrayals"]
            
            if recent_betrayals > 1:
                # High betrayal environment
                action_type = "betray" if random.random() > 0.7 else "catch"
                reasoning = f"High betrayal environment ({recent_betrayals} recent)"
                confidence = 0.6
            else:
                # Low betrayal, maintain trust
                action_type = "catch" if random.random() > 0.2 else "betray"
                reasoning = "Maintaining trust with occasional tests"
                confidence = 0.8
        
        return Action(
            action_type=action_type,
            data={},
            reasoning=reasoning,
            confidence=confidence
        )
    
    def update_history(self, round_result: RoundResult) -> None:
        """Update history with round results"""
        self.history.append(round_result)
        
        # Update opponent patterns
        for player_id, action in round_result.actions.items():
            if player_id != self.name:
                if player_id not in self.opponent_patterns:
                    self.opponent_patterns[player_id] = {}
                
                action_key = action.action_type
                if action_key not in self.opponent_patterns[player_id]:
                    self.opponent_patterns[player_id][action_key] = 0
                self.opponent_patterns[player_id][action_key] += 1
    
    def get_name(self) -> str:
        return self.name


class AIPlayer:
    """AI player that connects to the game server and plays autonomously"""
    
    def __init__(self, url: str, player_info: PlayerInfo, decision_maker: AIDecisionMaker, **options):
        """Initialize AI player"""
        self.client = GeniusGamesClient(url, player_info, **options)
        self.decision_maker = decision_maker
        self.player_info = player_info
        self.logger = logging.getLogger(f"AIPlayer-{player_info.name}")
        self.current_game_type: Optional[GameType] = None
        
        # Set up event handlers
        self._setup_handlers()
    
    def _setup_handlers(self) -> None:
        """Set up client event handlers"""
        self.client.on_game_state_update = self._on_game_state_update
        self.client.on_round_result = self._on_round_result
        self.client.on_game_ended = self._on_game_ended
        self.client.on_error = lambda error: self.logger.error(f"Client error: {error}")
    
    async def connect(self) -> None:
        """Connect to the game server"""
        await self.client.connect()
        self.logger.info(f"AI player {self.player_info.name} connected")
    
    async def disconnect(self) -> None:
        """Disconnect from the server"""
        await self.client.disconnect()
        self.logger.info(f"AI player {self.player_info.name} disconnected")
    
    async def create_and_play(self, game_config: Union[GameConfig, Dict[str, Any]]) -> GameResult:
        """Create a game and play it to completion"""
        # Extract game type
        if isinstance(game_config, dict):
            game_type_str = game_config.get("gameType", "")
            self.current_game_type = GameType(game_type_str) if game_type_str else None
        else:
            self.current_game_type = game_config.game_type if isinstance(game_config.game_type, GameType) else GameType(game_config.game_type)
        
        # Create game
        game_id = await self.client.create_game(game_config)
        self.logger.info(f"Created game {game_id}")
        
        # Wait for game to end
        game_result_future = asyncio.get_event_loop().create_future()
        self._game_result_future = game_result_future
        
        try:
            result = await game_result_future
            return result
        except Exception as e:
            self.logger.error(f"Game failed: {e}")
            raise
    
    async def join_and_play(self, game_id: str, game_type: GameType) -> GameResult:
        """Join an existing game and play it"""
        self.current_game_type = game_type
        await self.client.join_game(game_id)
        self.logger.info(f"Joined game {game_id}")
        
        # Wait for game to end
        game_result_future = asyncio.get_event_loop().create_future()
        self._game_result_future = game_result_future
        
        try:
            result = await game_result_future
            return result
        except Exception as e:
            self.logger.error(f"Game failed: {e}")
            raise
    
    def _on_game_state_update(self, game_state: GameState) -> None:
        """Handle game state updates"""
        asyncio.create_task(self._make_decision(game_state))
    
    async def _make_decision(self, game_state: GameState) -> None:
        """Make and submit a decision"""
        try:
            if not self.current_game_type:
                self.logger.warning("No game type set, cannot make decision")
                return
            
            # Get decision from AI
            action = await self.decision_maker.make_decision(game_state, self.current_game_type)
            
            # Submit action
            await self.client.submit_action(action)
            self.logger.debug(f"Submitted action: {action.action_type} - {action.reasoning}")
            
        except Exception as e:
            self.logger.error(f"Error making decision: {e}")
    
    def _on_round_result(self, round_result: RoundResult) -> None:
        """Handle round results"""
        # Update decision maker if it tracks history
        if isinstance(self.decision_maker, StrategicDecisionMaker):
            self.decision_maker.update_history(round_result)
        
        # Log round outcome
        self.logger.debug(f"Round {round_result.round} complete. Winners: {round_result.outcome.winners}")
    
    def _on_game_ended(self, game_result: GameResult) -> None:
        """Handle game end"""
        self.logger.info(f"Game ended. Winner: {game_result.winner}")
        
        # Complete the future
        if hasattr(self, '_game_result_future') and not self._game_result_future.done():
            self._game_result_future.set_result(game_result)