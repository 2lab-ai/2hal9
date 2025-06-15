/**
 * Game types available in the Genius Game Server
 */
export enum GameType {
  MinorityGame = 'MinorityGame',
  ByzantineGenerals = 'ByzantineGenerals',
  CollectiveMaze = 'CollectiveMaze',
  RecursiveReasoning = 'RecursiveReasoning',
  SwarmOptimization = 'SwarmOptimization',
  PrisonersDilemma = 'PrisonersDilemma',
  QuantumConsensus = 'QuantumConsensus',
  MiniGo = 'MiniGo',
  MiniHoldem = 'MiniHoldem',
  SquidGame = 'SquidGame',
  BattleRoyale = 'BattleRoyale',
  HungerGames = 'HungerGames',
  LiarsDice = 'LiarsDice',
  RussianRoulette = 'RussianRoulette',
  KingOfTheHill = 'KingOfTheHill',
  LastStand = 'LastStand',
  TrustFall = 'TrustFall',
}

/**
 * Game configuration
 */
export interface GameConfig {
  gameType: GameType;
  rounds: number;
  timeLimitMs: number;
  specialRules?: Record<string, string>;
}

/**
 * Game state
 */
export interface GameState {
  gameId: string;
  gameType: GameType;
  round: number;
  scores: Record<string, number>;
  history: RoundResult[];
  metadata: Record<string, any>;
}

/**
 * Player action
 */
export interface Action {
  playerId: string;
  actionType: string;
  data: any;
  reasoning?: string;
  confidence?: number;
}

/**
 * Round result
 */
export interface RoundResult {
  round: number;
  actions: Record<string, Action>;
  outcome: Outcome;
  scoresDelta: Record<string, number>;
  timestamp: string;
}

/**
 * Round outcome
 */
export interface Outcome {
  winners: string[];
  losers: string[];
  specialEvents: string[];
  emergenceDetected: boolean;
}

/**
 * Game result
 */
export interface GameResult {
  gameId: string;
  winner: string;
  finalScores: Record<string, number>;
  totalRounds: number;
  emergenceEvents: EmergenceEvent[];
  analytics: GameAnalytics;
}

/**
 * Emergence event
 */
export interface EmergenceEvent {
  round: number;
  eventType: string;
  description: string;
  emergenceScore: number;
}

/**
 * Game analytics
 */
export interface GameAnalytics {
  collectiveCoordinationScore: number;
  decisionDiversityIndex: number;
  strategicDepth: number;
  emergenceFrequency: number;
  performanceDifferential: number;
}

/**
 * WebSocket message types
 */
export enum MessageType {
  // Client -> Server
  JoinGame = 'join_game',
  CreateGame = 'create_game',
  SubmitAction = 'submit_action',
  LeaveGame = 'leave_game',
  
  // Server -> Client
  GameCreated = 'game_created',
  GameJoined = 'game_joined',
  GameStateUpdate = 'game_state_update',
  RoundResult = 'round_result',
  GameEnded = 'game_ended',
  Error = 'error',
}

/**
 * WebSocket message
 */
export interface WSMessage<T = any> {
  type: MessageType;
  gameId?: string;
  playerId?: string;
  data: T;
  timestamp: string;
}

/**
 * Connection options
 */
export interface ConnectionOptions {
  url: string;
  wsUrl?: string;
  reconnect?: boolean;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  debug?: boolean;
}

/**
 * Player info
 */
export interface PlayerInfo {
  id: string;
  name: string;
  type: 'human' | 'ai';
  aiProvider?: string;
  aiModel?: string;
}

/**
 * Event handlers
 */
export interface EventHandlers {
  onConnect?: () => void;
  onDisconnect?: (reason: string) => void;
  onGameCreated?: (gameId: string) => void;
  onGameJoined?: (gameState: GameState) => void;
  onGameStateUpdate?: (gameState: GameState) => void;
  onRoundResult?: (result: RoundResult) => void;
  onGameEnded?: (result: GameResult) => void;
  onError?: (error: Error) => void;
}

/**
 * AI decision maker interface
 */
export interface AIDecisionMaker {
  makeDecision(gameState: GameState, gameType: GameType): Promise<Action>;
  getName(): string;
}

/**
 * Game-specific action types
 */
export const GameActions = {
  [GameType.MinorityGame]: ['choose_0', 'choose_1'],
  [GameType.ByzantineGenerals]: ['attack', 'retreat'],
  [GameType.PrisonersDilemma]: ['cooperate', 'defect'],
  [GameType.QuantumConsensus]: ['measure', 'entangle'],
  [GameType.MiniGo]: ['place', 'pass'],
  [GameType.MiniHoldem]: ['fold', 'call', 'raise', 'check'],
  [GameType.SquidGame]: ['move', 'freeze'],
  [GameType.BattleRoyale]: ['move', 'attack', 'loot', 'hide'],
  [GameType.HungerGames]: ['hunt', 'gather', 'form_alliance', 'betray', 'move', 'use_item'],
  [GameType.LiarsDice]: ['bid', 'challenge'],
  [GameType.RussianRoulette]: ['spin', 'pull', 'pass'],
  [GameType.KingOfTheHill]: ['push', 'fortify', 'charge', 'defend'],
  [GameType.LastStand]: ['shoot', 'fortify', 'heal', 'share', 'scavenge'],
  [GameType.TrustFall]: ['fall', 'catch', 'betray', 'build_trust'],
} as const;