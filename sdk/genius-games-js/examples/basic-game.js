const { GeniusGamesClient, GameType } = require('@2lab/genius-games-sdk');

async function playMinorityGame() {
  // Create client
  const client = new GeniusGamesClient({
    url: 'http://localhost:8080',
    debug: true
  }, {
    id: 'player1',
    name: 'Alice',
    type: 'human'
  });

  try {
    // Connect to server
    console.log('Connecting to server...');
    await client.connect();
    console.log('Connected!');

    // Set up event handlers
    client.setHandlers({
      onGameStateUpdate: (gameState) => {
        console.log(`\n--- Round ${gameState.round} ---`);
        console.log('Current scores:', gameState.scores);
        
        // Make a decision based on the round number
        const choice = gameState.round % 2 === 0 ? 'choose_0' : 'choose_1';
        
        // Submit action
        client.submitAction({
          actionType: choice,
          data: {},
          reasoning: 'Alternating strategy',
          confidence: 0.5
        }).catch(console.error);
      },

      onRoundResult: (result) => {
        console.log('\nRound result:');
        console.log('Winners:', result.outcome.winners);
        console.log('My score delta:', result.scoresDelta['player1'] || 0);
        
        if (result.outcome.emergenceDetected) {
          console.log('🌟 Emergence detected!');
        }
      },

      onGameEnded: (result) => {
        console.log('\n=== GAME ENDED ===');
        console.log('Winner:', result.winner);
        console.log('Final scores:', result.finalScores);
        console.log('Total rounds:', result.totalRounds);
        
        // Disconnect after game ends
        client.disconnect();
      },

      onError: (error) => {
        console.error('Error:', error);
        client.disconnect();
      }
    });

    // Create a new game
    console.log('\nCreating Minority Game...');
    const gameId = await client.createGame({
      gameType: GameType.MinorityGame,
      rounds: 20,
      timeLimitMs: 5000
    });
    
    console.log('Game created with ID:', gameId);
    console.log('Waiting for other players to join...');

  } catch (error) {
    console.error('Failed to play game:', error);
    client.disconnect();
  }
}

// Run the example
playMinorityGame().catch(console.error);