import { EventEmitter } from 'eventemitter3';
import WebSocket from 'isomorphic-ws';
import {
  ConnectionOptions,
  EventHandlers,
  GameConfig,
  GameState,
  Action,
  WSMessage,
  MessageType,
  PlayerInfo,
  GameResult,
  RoundResult,
} from './types';

/**
 * Genius Games client for connecting to the game server
 */
export class GeniusGamesClient extends EventEmitter {
  private ws: WebSocket | null = null;
  private options: Required<ConnectionOptions>;
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  private currentGameId: string | null = null;
  private playerId: string;
  private playerInfo: PlayerInfo;
  private isConnected = false;

  constructor(options: ConnectionOptions, playerInfo: PlayerInfo) {
    super();
    
    this.options = {
      url: options.url,
      wsUrl: options.wsUrl || options.url.replace(/^http/, 'ws') + '/ws',
      reconnect: options.reconnect ?? true,
      reconnectInterval: options.reconnectInterval ?? 5000,
      maxReconnectAttempts: options.maxReconnectAttempts ?? 10,
      debug: options.debug ?? false,
    };
    
    this.playerId = playerInfo.id;
    this.playerInfo = playerInfo;
  }

  /**
   * Connect to the game server
   */
  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.log('Connecting to', this.options.wsUrl);
        
        this.ws = new WebSocket(this.options.wsUrl);
        
        this.ws.onopen = () => {
          this.log('Connected to server');
          this.isConnected = true;
          this.reconnectAttempts = 0;
          this.startHeartbeat();
          this.emit('connect');
          resolve();
        };
        
        this.ws.onmessage = (event) => {
          try {
            const message: WSMessage = JSON.parse(event.data.toString());
            this.handleMessage(message);
          } catch (error) {
            this.error('Failed to parse message:', error);
          }
        };
        
        this.ws.onerror = (error) => {
          this.error('WebSocket error:', error);
          this.emit('error', error);
          reject(error);
        };
        
        this.ws.onclose = (event) => {
          this.log('Disconnected from server', event.code, event.reason);
          this.isConnected = false;
          this.stopHeartbeat();
          this.emit('disconnect', event.reason);
          
          if (this.options.reconnect && this.reconnectAttempts < this.options.maxReconnectAttempts) {
            this.scheduleReconnect();
          }
        };
      } catch (error) {
        reject(error);
      }
    });
  }

  /**
   * Disconnect from the server
   */
  disconnect(): void {
    this.options.reconnect = false;
    this.stopHeartbeat();
    
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Create a new game
   */
  async createGame(config: GameConfig): Promise<string> {
    return new Promise((resolve, reject) => {
      if (!this.isConnected) {
        reject(new Error('Not connected to server'));
        return;
      }
      
      const handler = (gameId: string) => {
        this.off('gameCreated', handler);
        resolve(gameId);
      };
      
      this.once('gameCreated', handler);
      
      this.sendMessage({
        type: MessageType.CreateGame,
        playerId: this.playerId,
        data: config,
        timestamp: new Date().toISOString(),
      });
      
      // Timeout after 10 seconds
      setTimeout(() => {
        this.off('gameCreated', handler);
        reject(new Error('Create game timeout'));
      }, 10000);
    });
  }

  /**
   * Join an existing game
   */
  async joinGame(gameId: string): Promise<GameState> {
    return new Promise((resolve, reject) => {
      if (!this.isConnected) {
        reject(new Error('Not connected to server'));
        return;
      }
      
      const handler = (gameState: GameState) => {
        this.off('gameJoined', handler);
        resolve(gameState);
      };
      
      this.once('gameJoined', handler);
      
      this.currentGameId = gameId;
      this.sendMessage({
        type: MessageType.JoinGame,
        gameId,
        playerId: this.playerId,
        data: this.playerInfo,
        timestamp: new Date().toISOString(),
      });
      
      setTimeout(() => {
        this.off('gameJoined', handler);
        reject(new Error('Join game timeout'));
      }, 10000);
    });
  }

  /**
   * Submit an action for the current round
   */
  async submitAction(action: Omit<Action, 'playerId'>): Promise<void> {
    if (!this.isConnected) {
      throw new Error('Not connected to server');
    }
    
    if (!this.currentGameId) {
      throw new Error('Not in a game');
    }
    
    const fullAction: Action = {
      ...action,
      playerId: this.playerId,
    };
    
    this.sendMessage({
      type: MessageType.SubmitAction,
      gameId: this.currentGameId,
      playerId: this.playerId,
      data: fullAction,
      timestamp: new Date().toISOString(),
    });
  }

  /**
   * Leave the current game
   */
  async leaveGame(): Promise<void> {
    if (!this.currentGameId) {
      return;
    }
    
    this.sendMessage({
      type: MessageType.LeaveGame,
      gameId: this.currentGameId,
      playerId: this.playerId,
      data: null,
      timestamp: new Date().toISOString(),
    });
    
    this.currentGameId = null;
  }

  /**
   * Set event handlers
   */
  setHandlers(handlers: EventHandlers): void {
    if (handlers.onConnect) this.on('connect', handlers.onConnect);
    if (handlers.onDisconnect) this.on('disconnect', handlers.onDisconnect);
    if (handlers.onGameCreated) this.on('gameCreated', handlers.onGameCreated);
    if (handlers.onGameJoined) this.on('gameJoined', handlers.onGameJoined);
    if (handlers.onGameStateUpdate) this.on('gameStateUpdate', handlers.onGameStateUpdate);
    if (handlers.onRoundResult) this.on('roundResult', handlers.onRoundResult);
    if (handlers.onGameEnded) this.on('gameEnded', handlers.onGameEnded);
    if (handlers.onError) this.on('error', handlers.onError);
  }

  /**
   * Get current connection status
   */
  isConnectedToServer(): boolean {
    return this.isConnected;
  }

  /**
   * Get current game ID
   */
  getCurrentGameId(): string | null {
    return this.currentGameId;
  }

  private handleMessage(message: WSMessage): void {
    this.log('Received message:', message.type);
    
    switch (message.type) {
      case MessageType.GameCreated:
        this.currentGameId = message.data.gameId;
        this.emit('gameCreated', message.data.gameId);
        break;
        
      case MessageType.GameJoined:
        this.emit('gameJoined', message.data as GameState);
        break;
        
      case MessageType.GameStateUpdate:
        this.emit('gameStateUpdate', message.data as GameState);
        break;
        
      case MessageType.RoundResult:
        this.emit('roundResult', message.data as RoundResult);
        break;
        
      case MessageType.GameEnded:
        this.emit('gameEnded', message.data as GameResult);
        this.currentGameId = null;
        break;
        
      case MessageType.Error:
        this.error('Server error:', message.data);
        this.emit('error', new Error(message.data.message || 'Unknown error'));
        break;
        
      default:
        this.log('Unknown message type:', message.type);
    }
  }

  private sendMessage(message: WSMessage): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket not connected');
    }
    
    const data = JSON.stringify(message);
    this.log('Sending message:', message.type);
    this.ws.send(data);
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer) {
      return;
    }
    
    this.reconnectAttempts++;
    const delay = this.options.reconnectInterval * this.reconnectAttempts;
    
    this.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
    
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect().catch((error) => {
        this.error('Reconnection failed:', error);
      });
    }, delay);
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    
    this.heartbeatTimer = setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.ping();
      }
    }, 30000);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  private log(...args: any[]): void {
    if (this.options.debug) {
      console.log('[GeniusGamesClient]', ...args);
    }
  }

  private error(...args: any[]): void {
    console.error('[GeniusGamesClient]', ...args);
  }
}