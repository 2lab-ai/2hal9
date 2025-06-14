# AI Genius Game Competitions

This directory contains competition-related files for the HAL9 vs SOTA AI Genius Games.

## Structure

- `AI_GENIUS_GAME_2025.md` - Main competition documentation
- `GENIUS_GAME_COMPLETE_RULEBOOK.md` - Complete rulebook with 25 games
- `GENIUS_GAME_TECHNICAL_IMPLEMENTATION.md` - Technical implementation guide
- `genius_game_server/` - Rust-based game server implementation
- `game_interface*.html` - Demo visualization interfaces (HTML/JS for browser viewing only)

## Important Notes

The HTML files in this directory contain JavaScript for visualization purposes only. They are:
- NOT part of the core HAL9 implementation
- Used only for demonstrating game concepts in a web browser
- Separate from the actual Rust-based game server

All actual game logic and AI implementations are written in Rust within the `genius_game_server/` directory.