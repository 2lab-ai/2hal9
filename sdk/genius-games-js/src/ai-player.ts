import { GeniusGamesClient } from './client';
import {
  AIDecisionMaker,
  GameState,
  GameType,
  Action,
  PlayerInfo,
  ConnectionOptions,
  GameConfig,
  GameActions,
} from './types';

/**
 * AI Player that can automatically play games
 */
export class AIPlayer {
  private client: GeniusGamesClient;
  private decisionMaker: AIDecisionMaker;
  private playerInfo: PlayerInfo;
  private isPlaying = false;
  private currentGameState: GameState | null = null;

  constructor(
    connectionOptions: ConnectionOptions,
    playerInfo: PlayerInfo,
    decisionMaker: AIDecisionMaker
  ) {
    this.playerInfo = playerInfo;
    this.decisionMaker = decisionMaker;
    this.client = new GeniusGamesClient(connectionOptions, playerInfo);
    
    this.setupEventHandlers();
  }

  /**
   * Connect to the server
   */
  async connect(): Promise<void> {
    await this.client.connect();
  }

  /**
   * Disconnect from the server
   */
  disconnect(): void {
    this.isPlaying = false;
    this.client.disconnect();
  }

  /**
   * Create and play a new game
   */
  async createAndPlay(config: GameConfig): Promise<void> {
    const gameId = await this.client.createGame(config);
    await this.playGame(gameId);
  }

  /**
   * Join and play an existing game
   */
  async joinAndPlay(gameId: string): Promise<void> {
    await this.client.joinGame(gameId);
    await this.playGame(gameId);
  }

  /**
   * Start playing the current game
   */
  private async playGame(gameId: string): Promise<void> {
    this.isPlaying = true;
    console.log(`AI Player ${this.playerInfo.name} started playing game ${gameId}`);
  }

  /**
   * Stop playing
   */
  stopPlaying(): void {
    this.isPlaying = false;
    if (this.client.getCurrentGameId()) {
      this.client.leaveGame();
    }
  }

  private setupEventHandlers(): void {
    this.client.setHandlers({
      onConnect: () => {
        console.log(`AI Player ${this.playerInfo.name} connected`);
      },
      
      onDisconnect: (reason) => {
        console.log(`AI Player ${this.playerInfo.name} disconnected: ${reason}`);
        this.isPlaying = false;
      },
      
      onGameJoined: (gameState) => {
        console.log(`AI Player ${this.playerInfo.name} joined game ${gameState.gameId}`);
        this.currentGameState = gameState;
      },
      
      onGameStateUpdate: async (gameState) => {
        this.currentGameState = gameState;
        
        if (!this.isPlaying) {
          return;
        }
        
        try {
          // Make decision and submit action
          const action = await this.decisionMaker.makeDecision(gameState, gameState.gameType);
          await this.client.submitAction(action);
          
          console.log(`AI Player ${this.playerInfo.name} submitted action:`, action.actionType);
        } catch (error) {
          console.error(`AI Player ${this.playerInfo.name} failed to make decision:`, error);
        }
      },
      
      onRoundResult: (result) => {
        const myAction = result.actions[this.playerInfo.id];
        const myScoreDelta = result.scoresDelta[this.playerInfo.id] || 0;
        
        console.log(
          `Round ${result.round} result - Action: ${myAction?.actionType}, Score: ${myScoreDelta > 0 ? '+' : ''}${myScoreDelta}`
        );
      },
      
      onGameEnded: (result) => {
        this.isPlaying = false;
        const myScore = result.finalScores[this.playerInfo.id] || 0;
        const isWinner = result.winner === this.playerInfo.id;
        
        console.log(
          `Game ended - ${isWinner ? 'WON!' : 'Lost'} Final score: ${myScore}`
        );
      },
      
      onError: (error) => {
        console.error(`AI Player ${this.playerInfo.name} error:`, error);
      },
    });
  }

  /**
   * Get current game state
   */
  getGameState(): GameState | null {
    return this.currentGameState;
  }

  /**
   * Check if currently playing
   */
  isCurrentlyPlaying(): boolean {
    return this.isPlaying;
  }
}

/**
 * Simple random decision maker for testing
 */
export class RandomDecisionMaker implements AIDecisionMaker {
  private name: string;

  constructor(name = 'RandomAI') {
    this.name = name;
  }

  async makeDecision(gameState: GameState, gameType: GameType): Promise<Action> {
    const availableActions = GameActions[gameType] || ['default'];
    const randomAction = availableActions[Math.floor(Math.random() * availableActions.length)];
    
    return {
      playerId: '', // Will be filled by AIPlayer
      actionType: randomAction,
      data: this.generateActionData(randomAction, gameType),
      reasoning: `Random choice from ${availableActions.length} options`,
      confidence: 1 / availableActions.length,
    };
  }

  getName(): string {
    return this.name;
  }

  private generateActionData(actionType: string, gameType: GameType): any {
    // Generate appropriate data based on action type
    switch (gameType) {
      case GameType.MiniGo:
        if (actionType === 'place') {
          return {
            row: Math.floor(Math.random() * 9),
            col: Math.floor(Math.random() * 9),
          };
        }
        break;
        
      case GameType.MiniHoldem:
        if (actionType === 'raise') {
          return { amount: Math.floor(Math.random() * 50) + 10 };
        }
        break;
        
      case GameType.BattleRoyale:
        if (actionType === 'move') {
          const directions = ['North', 'South', 'East', 'West'];
          return directions[Math.floor(Math.random() * directions.length)];
        }
        break;
        
      case GameType.LiarsDice:
        if (actionType === 'bid') {
          return {
            quantity: Math.floor(Math.random() * 10) + 1,
            faceValue: Math.floor(Math.random() * 6) + 1,
          };
        }
        break;
    }
    
    return {};
  }
}

/**
 * Strategic decision maker that analyzes game state
 */
export class StrategicDecisionMaker implements AIDecisionMaker {
  private name: string;

  constructor(name = 'StrategicAI') {
    this.name = name;
  }

  async makeDecision(gameState: GameState, gameType: GameType): Promise<Action> {
    // Analyze game history
    const recentHistory = gameState.history.slice(-5);
    const myScore = gameState.scores[this.name] || 0;
    const avgScore = Object.values(gameState.scores).reduce((a, b) => a + b, 0) / Object.keys(gameState.scores).length;
    
    // Make strategic decision based on game type
    let action: Action;
    
    switch (gameType) {
      case GameType.PrisonersDilemma:
        action = this.decidePrisonersDilemma(recentHistory, myScore, avgScore);
        break;
        
      case GameType.MinorityGame:
        action = this.decideMinorityGame(recentHistory);
        break;
        
      case GameType.ByzantineGenerals:
        action = this.decideByzantineGenerals(gameState);
        break;
        
      default:
        // Fallback to random for unsupported games
        const randomMaker = new RandomDecisionMaker();
        action = await randomMaker.makeDecision(gameState, gameType);
    }
    
    return action;
  }

  getName(): string {
    return this.name;
  }

  private decidePrisonersDilemma(history: any[], myScore: number, avgScore: number): Action {
    // Tit-for-tat with forgiveness strategy
    let shouldCooperate = true;
    
    if (history.length > 0) {
      const lastRound = history[history.length - 1];
      const otherActions = Object.values(lastRound.actions).filter(
        (a: any) => a.playerId !== this.name
      );
      
      const defections = otherActions.filter((a: any) => a.actionType === 'defect').length;
      const cooperations = otherActions.filter((a: any) => a.actionType === 'cooperate').length;
      
      // Defect if majority defected, but forgive 20% of the time
      if (defections > cooperations && Math.random() > 0.2) {
        shouldCooperate = false;
      }
    }
    
    return {
      playerId: '',
      actionType: shouldCooperate ? 'cooperate' : 'defect',
      data: {},
      reasoning: shouldCooperate 
        ? 'Cooperating to build trust' 
        : 'Defecting in response to betrayal',
      confidence: 0.8,
    };
  }

  private decideMinorityGame(history: any[]): Action {
    // Anti-majority strategy with pattern detection
    const recentChoices = history.slice(-3).map(round => {
      const choices = Object.values(round.actions).map((a: any) => a.actionType);
      const zeros = choices.filter(c => c === 'choose_0').length;
      const ones = choices.filter(c => c === 'choose_1').length;
      return zeros > ones ? '0' : '1';
    });
    
    // If there's a pattern, break it
    let choice = 'choose_0';
    if (recentChoices.length >= 2) {
      if (recentChoices.every(c => c === '0')) {
        choice = 'choose_1';
      } else if (recentChoices.every(c => c === '1')) {
        choice = 'choose_0';
      } else {
        // Random if no clear pattern
        choice = Math.random() > 0.5 ? 'choose_0' : 'choose_1';
      }
    }
    
    return {
      playerId: '',
      actionType: choice,
      data: {},
      reasoning: 'Attempting to be in the minority',
      confidence: 0.6,
    };
  }

  private decideByzantineGenerals(gameState: GameState): Action {
    // Loyal general strategy - coordinate with majority
    const totalPlayers = Object.keys(gameState.scores).length;
    const expectedTraitors = Math.floor(totalPlayers * 0.33);
    
    // Default to attack unless we suspect too many traitors
    const shouldAttack = gameState.round < 5 || Object.keys(gameState.scores).length > expectedTraitors * 2;
    
    return {
      playerId: '',
      actionType: shouldAttack ? 'attack' : 'retreat',
      data: {},
      reasoning: shouldAttack 
        ? 'Coordinating attack with loyal generals' 
        : 'Too many potential traitors, retreating',
      confidence: 0.7,
    };
  }
}