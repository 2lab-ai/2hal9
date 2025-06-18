# 2HAL9 Project Structure Map

## Overview
The project has evolved into multiple directories with some duplication. Here's a comprehensive map:

## Main Project Structure

### `/Users/icedac/2lab.ai/2hal9/` (Main Project)
The primary workspace containing the current development.

#### Key Directories:
- **`competitions/`** - Competition-related code and documentation
  - `genius_game_server/` - Main game server implementation
  - HTML interfaces for games
  - Competition rules and technical documentation
  
- **`games/`** - Refactored game organization (newer structure)
  - `server/` - Game server with modular structure
  - `examples/` - Example implementations
  - `visualizations/` - Game visualization tools
  
- **`demo/`** - Demo scripts and utilities
  - Shell scripts for running various demos
  - Performance benchmarks
  - Visual demo tools

- **`layers/`** - AI layer implementations
- **`membrane/`** - Communication layer
- **`sdk/`** - SDKs for different languages
  - `genius-games-js/` - JavaScript SDK
  - `genius-games-py/` - Python SDK

### `/Users/icedac/2lab.ai/2hal9-demo/` (Demo/Showcase Project)
A separate repository focused on demonstrations and showcases.

#### Contains:
- **`demo/`** - HTML showcases and visualizations
  - Premium game showcases
  - Death game demos
  - Game launchers
  
- **`crates/genius-games/`** - Game implementations
  - Various game categories (strategic, survival, trust, consciousness)
  - E2E tests for each category

## Duplication Analysis

### 1. Game Implementations
**Complete Duplication Found:**
- `competitions/genius_game_server/src/games/` 
- `games/server/src/games/`

Both directories contain identical game files:
- battle_royale.rs
- byzantine_generals.rs
- collective_maze.rs
- death_game_tests.rs
- hunger_games.rs
- king_of_the_hill.rs
- last_stand.rs
- liars_dice.rs
- mini_go.rs
- mini_holdem.rs
- minority_game.rs
- prisoners_dilemma.rs
- quantum_consensus.rs
- recursive_reasoning.rs
- russian_roulette.rs
- squid_game.rs
- swarm_optimization.rs
- trust_fall.rs

### 2. Demo Files
**Partial Overlap:**
- Main project `demo/` - Contains shell scripts for running demos
- 2hal9-demo `demo/` - Contains HTML visualizations and game launchers

### 3. Server Structure
**Similar but Different:**
- `competitions/genius_game_server/` - Original implementation
- `games/server/` - Refactored, more modular structure with:
  - Separated AI providers
  - Analytics module
  - Streaming capabilities
  - Tournament system

## Recommendations

1. **Primary Development Location**: `games/server/` appears to be the newer, more organized structure
2. **Legacy Code**: `competitions/genius_game_server/` seems to be the original implementation
3. **Demo Repository**: `2hal9-demo` serves as a separate showcase/demo project
4. **Consolidation Needed**: The duplicate game implementations should be consolidated

## Directory Purposes

- **`competitions/`**: Competition documentation and original server
- **`games/`**: Refactored, production-ready game server
- **`demo/`**: Quick demo scripts and benchmarks
- **`2hal9-demo/`**: Public-facing demos and showcases