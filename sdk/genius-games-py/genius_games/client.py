"""
WebSocket client for Genius Games
"""

import asyncio
import json
import logging
from typing import Dict, Any, Optional, Callable, Union
from datetime import datetime
import websocket
from websocket import WebSocketApp
import threading
import queue

from .types import (
    ConnectionOptions,
    PlayerInfo,
    GameConfig,
    GameState,
    GameType,
    Action,
    RoundResult,
    GameResult,
    MessageType,
    PlayerState,
    RoundOutcome
)


class GeniusGamesClient:
    """WebSocket client for Genius Games server"""
    
    def __init__(self, url: str, player_info: PlayerInfo, **options):
        """Initialize client with connection options"""
        self.options = ConnectionOptions(
            url=url.replace("http://", "ws://").replace("https://", "wss://"),
            player_info=player_info,
            **options
        )
        
        self.ws: Optional[WebSocketApp] = None
        self.connected = False
        self.game_id: Optional[str] = None
        self.current_state: Optional[GameState] = None
        self._message_queue = queue.Queue()
        self._response_futures: Dict[str, asyncio.Future] = {}
        self._reconnect_count = 0
        self._ws_thread: Optional[threading.Thread] = None
        
        # Event handlers
        self.on_connect: Optional[Callable[[], None]] = None
        self.on_disconnect: Optional[Callable[[str], None]] = None
        self.on_game_created: Optional[Callable[[str], None]] = None
        self.on_game_joined: Optional[Callable[[str], None]] = None
        self.on_game_state_update: Optional[Callable[[GameState], None]] = None
        self.on_round_result: Optional[Callable[[RoundResult], None]] = None
        self.on_game_ended: Optional[Callable[[GameResult], None]] = None
        self.on_error: Optional[Callable[[str], None]] = None
        
        # Set up logging
        self.logger = logging.getLogger(__name__)
        if self.options.debug:
            self.logger.setLevel(logging.DEBUG)
    
    async def connect(self) -> None:
        """Connect to the game server"""
        if self.connected:
            return
            
        loop = asyncio.get_event_loop()
        connect_future = loop.create_future()
        
        def on_open(ws):
            self.connected = True
            self._reconnect_count = 0
            self.logger.info(f"Connected to {self.options.url}")
            
            # Send player info
            self._send_message({
                "type": "player_info",
                "data": {
                    "id": self.options.player_info.id,
                    "name": self.options.player_info.name,
                    "type": self.options.player_info.type
                }
            })
            
            if self.on_connect:
                self.on_connect()
            
            loop.call_soon_threadsafe(connect_future.set_result, None)
        
        def on_message(ws, message):
            try:
                data = json.loads(message)
                self._handle_message(data)
            except Exception as e:
                self.logger.error(f"Error handling message: {e}")
        
        def on_error(ws, error):
            self.logger.error(f"WebSocket error: {error}")
            if self.on_error:
                self.on_error(str(error))
            
            if not connect_future.done():
                loop.call_soon_threadsafe(connect_future.set_exception, error)
        
        def on_close(ws, close_status_code, close_msg):
            self.connected = False
            reason = f"Connection closed: {close_status_code} - {close_msg}"
            self.logger.info(reason)
            
            if self.on_disconnect:
                self.on_disconnect(reason)
            
            # Handle reconnection
            if self.options.reconnect and self._reconnect_count < self.options.max_reconnects:
                self._reconnect_count += 1
                self.logger.info(f"Reconnecting... (attempt {self._reconnect_count})")
                asyncio.create_task(self._reconnect())
        
        # Create WebSocket connection
        self.ws = WebSocketApp(
            self.options.url,
            on_open=on_open,
            on_message=on_message,
            on_error=on_error,
            on_close=on_close,
            header=self.options.headers
        )
        
        # Run WebSocket in separate thread
        self._ws_thread = threading.Thread(target=self.ws.run_forever)
        self._ws_thread.daemon = True
        self._ws_thread.start()
        
        # Wait for connection
        try:
            await asyncio.wait_for(connect_future, timeout=self.options.timeout / 1000)
        except asyncio.TimeoutError:
            raise Exception("Connection timeout")
    
    async def disconnect(self) -> None:
        """Disconnect from the server"""
        if self.ws:
            self.ws.close()
            self.connected = False
            if self._ws_thread:
                self._ws_thread.join(timeout=5)
    
    async def create_game(self, config: Union[GameConfig, Dict[str, Any]]) -> str:
        """Create a new game"""
        if isinstance(config, dict):
            game_config = config
        else:
            game_config = {
                "gameType": config.game_type.value if isinstance(config.game_type, GameType) else config.game_type,
                "rounds": config.rounds,
                "timeLimitMs": config.time_limit_ms,
                "metadata": config.metadata
            }
        
        response = await self._send_and_wait({
            "type": MessageType.CREATE_GAME.value,
            "data": game_config
        })
        
        self.game_id = response["gameId"]
        return self.game_id
    
    async def join_game(self, game_id: str) -> None:
        """Join an existing game"""
        await self._send_and_wait({
            "type": MessageType.JOIN_GAME.value,
            "data": {"gameId": game_id}
        })
        
        self.game_id = game_id
    
    async def submit_action(self, action: Union[Action, Dict[str, Any]]) -> None:
        """Submit an action for the current round"""
        if not self.game_id:
            raise Exception("Not in a game")
        
        if isinstance(action, dict):
            action_data = action
        else:
            action_data = {
                "actionType": action.action_type,
                "data": action.data,
                "reasoning": action.reasoning,
                "confidence": action.confidence
            }
        
        self._send_message({
            "type": MessageType.SUBMIT_ACTION.value,
            "gameId": self.game_id,
            "data": action_data
        })
    
    async def leave_game(self) -> None:
        """Leave the current game"""
        if self.game_id:
            self._send_message({
                "type": MessageType.LEAVE_GAME.value,
                "gameId": self.game_id
            })
            self.game_id = None
    
    def get_current_state(self) -> Optional[GameState]:
        """Get the current game state"""
        return self.current_state
    
    def _send_message(self, message: Dict[str, Any]) -> None:
        """Send a message to the server"""
        if self.ws and self.connected:
            self.ws.send(json.dumps(message))
            if self.options.debug:
                self.logger.debug(f"Sent: {message}")
    
    async def _send_and_wait(self, message: Dict[str, Any], timeout: float = 10) -> Dict[str, Any]:
        """Send a message and wait for response"""
        request_id = f"{message['type']}_{datetime.now().timestamp()}"
        message["requestId"] = request_id
        
        future = asyncio.get_event_loop().create_future()
        self._response_futures[request_id] = future
        
        self._send_message(message)
        
        try:
            return await asyncio.wait_for(future, timeout=timeout)
        except asyncio.TimeoutError:
            del self._response_futures[request_id]
            raise Exception(f"Request timeout: {message['type']}")
    
    def _handle_message(self, data: Dict[str, Any]) -> None:
        """Handle incoming WebSocket message"""
        msg_type = data.get("type")
        
        if self.options.debug:
            self.logger.debug(f"Received: {data}")
        
        # Handle response to request
        request_id = data.get("requestId")
        if request_id and request_id in self._response_futures:
            future = self._response_futures.pop(request_id)
            if not future.done():
                asyncio.get_event_loop().call_soon_threadsafe(
                    future.set_result, data.get("data", {})
                )
            return
        
        # Handle different message types
        if msg_type == MessageType.GAME_CREATED.value:
            game_id = data["data"]["gameId"]
            self.game_id = game_id
            if self.on_game_created:
                self.on_game_created(game_id)
        
        elif msg_type == MessageType.GAME_JOINED.value:
            if self.on_game_joined:
                self.on_game_joined(data["data"]["gameId"])
        
        elif msg_type == MessageType.GAME_STATE_UPDATE.value:
            self.current_state = self._parse_game_state(data["data"])
            if self.on_game_state_update:
                self.on_game_state_update(self.current_state)
        
        elif msg_type == MessageType.ROUND_RESULT.value:
            result = self._parse_round_result(data["data"])
            if self.on_round_result:
                self.on_round_result(result)
        
        elif msg_type == MessageType.GAME_ENDED.value:
            result = self._parse_game_result(data["data"])
            self.game_id = None
            self.current_state = None
            if self.on_game_ended:
                self.on_game_ended(result)
        
        elif msg_type == MessageType.ERROR.value:
            error = data["data"].get("message", "Unknown error")
            self.logger.error(f"Server error: {error}")
            if self.on_error:
                self.on_error(error)
    
    def _parse_game_state(self, data: Dict[str, Any]) -> GameState:
        """Parse game state from server data"""
        players = []
        for p in data.get("players", []):
            players.append(PlayerState(
                id=p["id"],
                name=p["name"],
                score=p.get("score", 0),
                alive=p.get("alive", True),
                metadata=p.get("metadata", {})
            ))
        
        return GameState(
            game_id=data["gameId"],
            game_type=data["gameType"],
            round=data["round"],
            max_rounds=data["maxRounds"],
            players=players,
            scores=data.get("scores", {}),
            alive_players=data.get("alivePlayers", []),
            current_player=data.get("currentPlayer"),
            phase=data.get("phase"),
            metadata=data.get("metadata", {})
        )
    
    def _parse_round_result(self, data: Dict[str, Any]) -> RoundResult:
        """Parse round result from server data"""
        actions = {}
        for player_id, action_data in data.get("actions", {}).items():
            actions[player_id] = Action(
                action_type=action_data["actionType"],
                data=action_data.get("data", {}),
                reasoning=action_data.get("reasoning"),
                confidence=action_data.get("confidence", 1.0)
            )
        
        outcome = RoundOutcome(
            winners=data["outcome"]["winners"],
            losers=data["outcome"]["losers"],
            eliminated=data["outcome"].get("eliminated", []),
            scores=data["outcome"].get("scores", {}),
            metadata=data["outcome"].get("metadata", {})
        )
        
        return RoundResult(
            game_id=data["gameId"],
            round=data["round"],
            actions=actions,
            outcome=outcome,
            state=self._parse_game_state(data["state"])
        )
    
    def _parse_game_result(self, data: Dict[str, Any]) -> GameResult:
        """Parse game result from server data"""
        return GameResult(
            game_id=data["gameId"],
            game_type=data["gameType"],
            winner=data.get("winner"),
            winners=data.get("winners", []),
            final_scores=data["finalScores"],
            rounds_played=data["roundsPlayed"],
            start_time=datetime.fromisoformat(data["startTime"]),
            end_time=datetime.fromisoformat(data["endTime"]),
            metadata=data.get("metadata", {})
        )
    
    async def _reconnect(self) -> None:
        """Attempt to reconnect to the server"""
        await asyncio.sleep(self.options.reconnect_interval / 1000)
        try:
            await self.connect()
            if self.game_id:
                await self.join_game(self.game_id)
        except Exception as e:
            self.logger.error(f"Reconnection failed: {e}")