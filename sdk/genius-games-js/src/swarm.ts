import { AIPlayer, RandomDecisionMaker, StrategicDecisionMaker } from './ai-player';
import { ConnectionOptions, PlayerInfo, AIDecisionMaker, GameConfig } from './types';

/**
 * Options for creating an AI swarm
 */
export interface SwarmOptions {
  size: number;
  namePrefix?: string;
  decisionMaker?: 'random' | 'strategic' | AIDecisionMaker;
  aiProvider?: string;
  aiModel?: string;
}

/**
 * AI Swarm manager for running multiple AI players
 */
export class AISwarm {
  private players: AIPlayer[] = [];
  private connected = false;

  constructor(
    private connectionOptions: ConnectionOptions,
    private swarmOptions: SwarmOptions
  ) {}

  /**
   * Initialize and connect all AI players
   */
  async connect(): Promise<void> {
    const promises: Promise<void>[] = [];
    
    for (let i = 0; i < this.swarmOptions.size; i++) {
      const playerInfo: PlayerInfo = {
        id: `${this.swarmOptions.namePrefix || 'swarm'}_${i}`,
        name: `${this.swarmOptions.namePrefix || 'Swarm'}-${i}`,
        type: 'ai',
        aiProvider: this.swarmOptions.aiProvider,
        aiModel: this.swarmOptions.aiModel,
      };
      
      const decisionMaker = this.createDecisionMaker(i);
      const player = new AIPlayer(this.connectionOptions, playerInfo, decisionMaker);
      
      this.players.push(player);
      promises.push(player.connect());
    }
    
    await Promise.all(promises);
    this.connected = true;
    
    console.log(`AI Swarm connected with ${this.players.length} players`);
  }

  /**
   * Disconnect all players
   */
  disconnect(): void {
    this.players.forEach(player => player.disconnect());
    this.connected = false;
    console.log('AI Swarm disconnected');
  }

  /**
   * Create and play a new game with all swarm members
   */
  async createAndPlayGame(config: GameConfig): Promise<void> {
    if (!this.connected) {
      throw new Error('Swarm not connected');
    }
    
    if (this.players.length === 0) {
      throw new Error('No players in swarm');
    }
    
    // First player creates the game
    const creator = this.players[0];
    await creator.createAndPlay(config);
    
    // Get the game ID
    const gameId = creator['client'].getCurrentGameId();
    if (!gameId) {
      throw new Error('Failed to create game');
    }
    
    // Other players join
    const joinPromises = this.players.slice(1).map(player => 
      player.joinAndPlay(gameId)
    );
    
    await Promise.all(joinPromises);
    
    console.log(`Swarm started playing game ${gameId}`);
  }

  /**
   * Join an existing game with all swarm members
   */
  async joinGame(gameId: string): Promise<void> {
    if (!this.connected) {
      throw new Error('Swarm not connected');
    }
    
    const joinPromises = this.players.map(player => 
      player.joinAndPlay(gameId)
    );
    
    await Promise.all(joinPromises);
    
    console.log(`Swarm joined game ${gameId}`);
  }

  /**
   * Stop all players from playing
   */
  stopPlaying(): void {
    this.players.forEach(player => player.stopPlaying());
  }

  /**
   * Get swarm statistics
   */
  getStats(): {
    totalPlayers: number;
    activePlayers: number;
    connected: boolean;
  } {
    const activePlayers = this.players.filter(p => p.isCurrentlyPlaying()).length;
    
    return {
      totalPlayers: this.players.length,
      activePlayers,
      connected: this.connected,
    };
  }

  private createDecisionMaker(index: number): AIDecisionMaker {
    if (typeof this.swarmOptions.decisionMaker === 'object') {
      return this.swarmOptions.decisionMaker;
    }
    
    const name = `${this.swarmOptions.namePrefix || 'Swarm'}-${index}`;
    
    switch (this.swarmOptions.decisionMaker) {
      case 'strategic':
        return new StrategicDecisionMaker(name);
      case 'random':
      default:
        return new RandomDecisionMaker(name);
    }
  }
}

/**
 * Create an AI swarm with the given options
 */
export async function createAISwarm(
  connectionOptions: ConnectionOptions,
  swarmOptions: SwarmOptions
): Promise<AISwarm> {
  const swarm = new AISwarm(connectionOptions, swarmOptions);
  await swarm.connect();
  return swarm;
}