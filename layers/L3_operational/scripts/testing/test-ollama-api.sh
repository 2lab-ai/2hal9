#!/bin/bash

echo "🧪 Testing Ollama with Genius Game Server"
echo "========================================"
echo ""

# First, create a SOTA player with Ollama
echo "1️⃣ Creating Ollama SOTA player..."
curl -X POST http://localhost:8080/api/player/sota/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ollama_master",
    "ai_models": [
      {
        "provider": "ollama",
        "model": "llama3:latest",
        "endpoint": "http://localhost:11434"
      }
    ],
    "thinking_time": {
      "min_ms": 100,
      "max_ms": 5000,
      "strategy": "adaptive"
    }
  }' | jq '.'

echo ""
echo "2️⃣ Creating a game with Ollama player..."
curl -X POST http://localhost:8080/api/game/create \
  -H "Content-Type: application/json" \
  -d '{
    "game_type": "prisoners_dilemma",
    "rounds": 5,
    "time_limit_ms": 10000,
    "collective_players": [],
    "sota_players": ["ollama_master", "random_bot"]
  }' | jq '.'

echo ""
echo "✅ Test complete!"