import { GeniusGamesClient } from './client';
import { AISwarm } from './swarm';
import { 
  ConnectionOptions,
  GameConfig,
  GameType,
  GameResult,
  PlayerInfo,
} from './types';

/**
 * Options for running a simulation
 */
export interface SimulationOptions {
  games: GameConfig[];
  swarmSize: number;
  swarmType?: 'random' | 'strategic' | 'mixed';
  rounds?: number;
  delayBetweenGames?: number;
  verbose?: boolean;
}

/**
 * Simulation results
 */
export interface SimulationResults {
  totalGames: number;
  completedGames: number;
  failedGames: number;
  results: GameResult[];
  winRates: Record<string, number>;
  avgGameDuration: number;
  avgScores: Record<string, number>;
  emergenceEvents: number;
}

/**
 * Game simulator for running experiments
 */
export class GameSimulator {
  private connectionOptions: ConnectionOptions;
  private humanClient?: GeniusGamesClient;

  constructor(connectionOptions: ConnectionOptions) {
    this.connectionOptions = connectionOptions;
  }

  /**
   * Run a simulation with AI players
   */
  async runSimulation(options: SimulationOptions): Promise<SimulationResults> {
    const results: SimulationResults = {
      totalGames: options.games.length * (options.rounds || 1),
      completedGames: 0,
      failedGames: 0,
      results: [],
      winRates: {},
      avgGameDuration: 0,
      avgScores: {},
      emergenceEvents: 0,
    };
    
    // Create AI swarm
    const swarm = new AISwarm(this.connectionOptions, {
      size: options.swarmSize,
      namePrefix: 'SimBot',
      decisionMaker: options.swarmType === 'mixed' ? 'random' : options.swarmType || 'random',
    });
    
    try {
      await swarm.connect();
      
      if (options.verbose) {
        console.log(`Starting simulation with ${options.swarmSize} AI players`);
      }
      
      // Run games
      for (let round = 0; round < (options.rounds || 1); round++) {
        for (const gameConfig of options.games) {
          if (options.verbose) {
            console.log(`\nRound ${round + 1}: Starting ${gameConfig.gameType}`);
          }
          
          try {
            const gameResult = await this.runSingleGame(swarm, gameConfig, options);
            results.results.push(gameResult);
            results.completedGames++;
            
            // Update statistics
            this.updateStats(results, gameResult);
            
            if (options.verbose) {
              console.log(`Game completed. Winner: ${gameResult.winner}`);
            }
          } catch (error) {
            results.failedGames++;
            if (options.verbose) {
              console.error('Game failed:', error);
            }
          }
          
          // Delay between games
          if (options.delayBetweenGames) {
            await new Promise(resolve => setTimeout(resolve, options.delayBetweenGames));
          }
        }
      }
      
      // Calculate final statistics
      this.calculateFinalStats(results);
      
    } finally {
      swarm.disconnect();
    }
    
    return results;
  }

  /**
   * Run a single game and observe as human
   */
  async observeGame(gameConfig: GameConfig, swarmSize: number): Promise<GameResult> {
    // Create human observer client
    const playerInfo: PlayerInfo = {
      id: 'human_observer',
      name: 'Observer',
      type: 'human',
    };
    
    this.humanClient = new GeniusGamesClient(this.connectionOptions, playerInfo);
    
    // Create AI swarm
    const swarm = new AISwarm(this.connectionOptions, {
      size: swarmSize,
      namePrefix: 'AI',
      decisionMaker: 'strategic',
    });
    
    try {
      // Connect everyone
      await Promise.all([
        this.humanClient.connect(),
        swarm.connect(),
      ]);
      
      // Create game
      const gameId = await this.humanClient.createGame(gameConfig);
      console.log(`Created game ${gameId}`);
      
      // Setup observation handlers
      return new Promise<GameResult>((resolve, reject) => {
        this.humanClient!.setHandlers({
          onGameStateUpdate: (state) => {
            console.log(`\nRound ${state.round}`);
            console.log('Scores:', state.scores);
          },
          
          onRoundResult: (result) => {
            console.log('Round result:', result.outcome.specialEvents);
            if (result.outcome.emergenceDetected) {
              console.log('🌟 Emergence detected!');
            }
          },
          
          onGameEnded: (result) => {
            console.log('\nGame ended!');
            console.log('Winner:', result.winner);
            console.log('Final scores:', result.finalScores);
            resolve(result);
          },
          
          onError: (error) => {
            reject(error);
          },
        });
        
        // AI swarm joins the game
        swarm.joinGame(gameId);
      });
      
    } finally {
      if (this.humanClient) {
        this.humanClient.disconnect();
      }
      swarm.disconnect();
    }
  }

  private async runSingleGame(
    swarm: AISwarm,
    gameConfig: GameConfig,
    options: SimulationOptions
  ): Promise<GameResult> {
    return new Promise<GameResult>((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Game timeout'));
      }, 300000); // 5 minute timeout
      
      // Create temporary client to monitor game
      const monitorInfo: PlayerInfo = {
        id: 'monitor',
        name: 'Monitor',
        type: 'ai',
      };
      
      const monitor = new GeniusGamesClient(this.connectionOptions, monitorInfo);
      
      monitor.setHandlers({
        onGameEnded: (result) => {
          clearTimeout(timeout);
          monitor.disconnect();
          resolve(result);
        },
        
        onError: (error) => {
          clearTimeout(timeout);
          monitor.disconnect();
          reject(error);
        },
      });
      
      monitor.connect().then(() => {
        swarm.createAndPlayGame(gameConfig).catch(reject);
      }).catch(reject);
    });
  }

  private updateStats(results: SimulationResults, gameResult: GameResult): void {
    // Update win rates
    if (!results.winRates[gameResult.winner]) {
      results.winRates[gameResult.winner] = 0;
    }
    results.winRates[gameResult.winner]++;
    
    // Update average scores
    for (const [player, score] of Object.entries(gameResult.finalScores)) {
      if (!results.avgScores[player]) {
        results.avgScores[player] = 0;
      }
      results.avgScores[player] += score;
    }
    
    // Count emergence events
    results.emergenceEvents += gameResult.emergenceEvents.length;
    
    // Update average game duration
    results.avgGameDuration += gameResult.totalRounds;
  }

  private calculateFinalStats(results: SimulationResults): void {
    // Convert win counts to rates
    for (const player in results.winRates) {
      results.winRates[player] = results.winRates[player] / results.completedGames;
    }
    
    // Calculate average scores
    for (const player in results.avgScores) {
      results.avgScores[player] = results.avgScores[player] / results.completedGames;
    }
    
    // Calculate average game duration
    if (results.completedGames > 0) {
      results.avgGameDuration = results.avgGameDuration / results.completedGames;
    }
  }
}

/**
 * Run a quick simulation
 */
export async function quickSimulation(
  serverUrl: string,
  gameType: GameType,
  numPlayers: number = 4,
  numGames: number = 10
): Promise<SimulationResults> {
  const simulator = new GameSimulator({ url: serverUrl });
  
  return simulator.runSimulation({
    games: [{
      gameType,
      rounds: 20,
      timeLimitMs: 5000,
    }],
    swarmSize: numPlayers,
    swarmType: 'mixed',
    rounds: numGames,
    verbose: true,
  });
}