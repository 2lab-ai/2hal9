/**
 * Genius Games JavaScript/TypeScript SDK
 * 
 * A client library for connecting to the Genius Game Server
 * and playing AI collective intelligence games.
 */

// Main client
export { GeniusGamesClient } from './client';

// AI player helpers
export { AIPlayer, RandomDecisionMaker, StrategicDecisionMaker } from './ai-player';

// Types
export * from './types';

// Convenience functions
export { createAISwarm } from './swarm';
export { GameSimulator } from './simulator';

// Version
export const VERSION = '1.0.0';