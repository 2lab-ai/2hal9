#!/bin/bash

echo "🚀 AI Genius Game - Local Model Testing Script"
echo "=============================================="
echo ""

# Check if Ollama is installed
if ! command -v ollama &> /dev/null; then
    echo "❌ Ollama is not installed!"
    echo "Please install Ollama from: https://ollama.ai"
    exit 1
fi

# Check if Ollama is running
if ! curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo "⚠️  Ollama is not running. Starting Ollama..."
    ollama serve &
    sleep 3
fi

echo "✅ Ollama is running!"
echo ""

# List available models
echo "📦 Available models:"
ollama list

echo ""
echo "🎮 Running demo with local models..."
echo ""

# Run the demo
cargo run --bin demo_ollama

echo ""
echo "✅ Demo complete!"