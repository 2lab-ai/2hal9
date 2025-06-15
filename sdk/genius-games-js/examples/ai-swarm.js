const { createAISwarm, GameType, quickSimulation } = require('@2lab/genius-games-sdk');

async function runAISwarmDemo() {
  console.log('🤖 AI Swarm Demo - 16 Agents Playing Together\n');

  try {
    // Create a swarm of 16 AI agents
    console.log('Creating AI swarm...');
    const swarm = await createAISwarm(
      { 
        url: 'http://localhost:8080',
        debug: false // Set to true for detailed logs
      },
      {
        size: 16,
        namePrefix: 'SwarmBot',
        decisionMaker: 'strategic' // Use strategic AI
      }
    );

    console.log('✅ Swarm connected with 16 agents\n');

    // Play different games
    const games = [
      {
        type: GameType.MinorityGame,
        name: 'Minority Game',
        rounds: 30
      },
      {
        type: GameType.ByzantineGenerals,
        name: 'Byzantine Generals',
        rounds: 15
      },
      {
        type: GameType.PrisonersDilemma,
        name: "Prisoner's Dilemma",
        rounds: 20
      },
      {
        type: GameType.BattleRoyale,
        name: 'Battle Royale',
        rounds: 50
      }
    ];

    for (const game of games) {
      console.log(`\n🎮 Starting ${game.name}...`);
      
      try {
        await swarm.createAndPlayGame({
          gameType: game.type,
          rounds: game.rounds,
          timeLimitMs: 5000
        });

        // Wait for game to complete
        await new Promise(resolve => setTimeout(resolve, game.rounds * 1000));
        
        console.log(`✅ ${game.name} completed`);
        
        // Get swarm statistics
        const stats = swarm.getStats();
        console.log(`   Active players: ${stats.activePlayers}/${stats.totalPlayers}`);
        
      } catch (error) {
        console.error(`❌ ${game.name} failed:`, error.message);
      }

      // Brief pause between games
      await new Promise(resolve => setTimeout(resolve, 2000));
    }

    // Disconnect the swarm
    console.log('\n👋 Disconnecting swarm...');
    swarm.disconnect();
    console.log('✅ Demo completed!');

  } catch (error) {
    console.error('Demo failed:', error);
  }
}

async function runQuickSimulation() {
  console.log('\n📊 Running Quick Simulation...\n');

  try {
    const results = await quickSimulation(
      'http://localhost:8080',
      GameType.MinorityGame,
      8,  // 8 players
      10  // 10 games
    );

    console.log('\n=== SIMULATION RESULTS ===');
    console.log(`Total games: ${results.totalGames}`);
    console.log(`Completed: ${results.completedGames}`);
    console.log(`Failed: ${results.failedGames}`);
    console.log(`Average game duration: ${results.avgGameDuration.toFixed(1)} rounds`);
    console.log(`Emergence events: ${results.emergenceEvents}`);
    
    console.log('\nWin rates:');
    const sortedWinRates = Object.entries(results.winRates)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 5);
    
    for (const [player, rate] of sortedWinRates) {
      console.log(`  ${player}: ${(rate * 100).toFixed(1)}%`);
    }

    console.log('\nTop average scores:');
    const sortedScores = Object.entries(results.avgScores)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 5);
    
    for (const [player, score] of sortedScores) {
      console.log(`  ${player}: ${score.toFixed(1)}`);
    }

  } catch (error) {
    console.error('Simulation failed:', error);
  }
}

// Run both demos
async function main() {
  await runAISwarmDemo();
  await runQuickSimulation();
}

main().catch(console.error);