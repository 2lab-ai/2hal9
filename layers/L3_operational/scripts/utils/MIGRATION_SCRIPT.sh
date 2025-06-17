#!/bin/bash
# Migration script for Genius Game Server to 2hal9-demo

set -e

echo "🚀 Starting migration of Genius Game Server to 2hal9-demo..."

SOURCE_DIR="./competitions/genius_game_server"
TARGET_DIR="../2hal9-demo"

# Check if directories exist
if [ ! -d "$SOURCE_DIR" ]; then
    echo "❌ Source directory not found: $SOURCE_DIR"
    exit 1
fi

if [ ! -d "$TARGET_DIR" ]; then
    echo "❌ Target directory not found: $TARGET_DIR"
    exit 1
fi

echo "📁 Creating remaining crate structures..."

# Create remaining Cargo.toml files
cat > "$TARGET_DIR/crates/genius-engine/Cargo.toml" << 'EOF'
[package]
name = "genius-engine"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
genius-core = { path = "../genius-core" }
tokio = { workspace = true }
async-trait = { workspace = true }
dashmap = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
EOF

cat > "$TARGET_DIR/crates/genius-ai/Cargo.toml" << 'EOF'
[package]
name = "genius-ai"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
genius-core = { path = "../genius-core" }
async-trait = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
ollama-rs = { workspace = true }
aws-sdk-bedrockruntime = { workspace = true }
async-openai = { workspace = true }
dashmap = { workspace = true }
rand = { workspace = true }
EOF

cat > "$TARGET_DIR/crates/genius-games/Cargo.toml" << 'EOF'
[package]
name = "genius-games"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
genius-core = { path = "../genius-core" }
async-trait = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
rand = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
dashmap = { workspace = true }
EOF

cat > "$TARGET_DIR/crates/genius-server/Cargo.toml" << 'EOF'
[package]
name = "genius-server"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "genius-server"
path = "src/main.rs"

[dependencies]
genius-core = { path = "../genius-core" }
genius-engine = { path = "../genius-engine" }
genius-ai = { path = "../genius-ai" }
genius-games = { path = "../genius-games" }
tokio = { workspace = true }
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
dashmap = { workspace = true }
dotenvy = { workspace = true }
config = { workspace = true }
EOF

cat > "$TARGET_DIR/crates/genius-client/Cargo.toml" << 'EOF'
[package]
name = "genius-client"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
genius-core = { path = "../genius-core" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio-tungstenite = "0.24"
futures-util = "0.3"
EOF

echo "📦 Copying game implementations..."

# Copy strategic games
mkdir -p "$TARGET_DIR/crates/genius-games/src/strategic"
cp "$SOURCE_DIR/src/games/minority_game.rs" "$TARGET_DIR/crates/genius-games/src/strategic/"
cp "$SOURCE_DIR/src/games/byzantine_generals.rs" "$TARGET_DIR/crates/genius-games/src/strategic/"
cp "$SOURCE_DIR/src/games/mini_go.rs" "$TARGET_DIR/crates/genius-games/src/strategic/"
cp "$SOURCE_DIR/src/games/mini_holdem.rs" "$TARGET_DIR/crates/genius-games/src/strategic/"

# Copy collective games
mkdir -p "$TARGET_DIR/crates/genius-games/src/collective"
cp "$SOURCE_DIR/src/games/collective_maze.rs" "$TARGET_DIR/crates/genius-games/src/collective/"
cp "$SOURCE_DIR/src/games/swarm_optimization.rs" "$TARGET_DIR/crates/genius-games/src/collective/"
cp "$SOURCE_DIR/src/games/recursive_reasoning.rs" "$TARGET_DIR/crates/genius-games/src/collective/"
cp "$SOURCE_DIR/src/games/quantum_consensus.rs" "$TARGET_DIR/crates/genius-games/src/collective/"

# Copy survival games
mkdir -p "$TARGET_DIR/crates/genius-games/src/survival"
cp "$SOURCE_DIR/src/games/battle_royale.rs" "$TARGET_DIR/crates/genius-games/src/survival/"
cp "$SOURCE_DIR/src/games/hunger_games.rs" "$TARGET_DIR/crates/genius-games/src/survival/"
cp "$SOURCE_DIR/src/games/squid_game.rs" "$TARGET_DIR/crates/genius-games/src/survival/"
cp "$SOURCE_DIR/src/games/russian_roulette.rs" "$TARGET_DIR/crates/genius-games/src/survival/"
cp "$SOURCE_DIR/src/games/king_of_the_hill.rs" "$TARGET_DIR/crates/genius-games/src/survival/"
cp "$SOURCE_DIR/src/games/last_stand.rs" "$TARGET_DIR/crates/genius-games/src/survival/"

# Copy trust games
mkdir -p "$TARGET_DIR/crates/genius-games/src/trust"
cp "$SOURCE_DIR/src/games/prisoners_dilemma.rs" "$TARGET_DIR/crates/genius-games/src/trust/"
cp "$SOURCE_DIR/src/games/trust_fall.rs" "$TARGET_DIR/crates/genius-games/src/trust/"
cp "$SOURCE_DIR/src/games/liars_dice.rs" "$TARGET_DIR/crates/genius-games/src/trust/"

echo "🤖 Copying AI providers..."
cp -r "$SOURCE_DIR/src/ai_providers" "$TARGET_DIR/crates/genius-ai/src/"

echo "🚀 Copying engine components..."
cp "$SOURCE_DIR/src/analytics.rs" "$TARGET_DIR/crates/genius-engine/src/"
cp "$SOURCE_DIR/src/streaming.rs" "$TARGET_DIR/crates/genius-engine/src/"

echo "🌐 Copying server implementation..."
cp "$SOURCE_DIR/src/server.rs" "$TARGET_DIR/crates/genius-server/src/"
cp "$SOURCE_DIR/src/main.rs" "$TARGET_DIR/crates/genius-server/src/"

echo "📝 Copying demos and examples..."
cp -r "$SOURCE_DIR/demo" "$TARGET_DIR/"
cp -r "$SOURCE_DIR/examples" "$TARGET_DIR/"
cp -r "$SOURCE_DIR/public" "$TARGET_DIR/"

echo "🐳 Copying deployment files..."
cp "$SOURCE_DIR/Dockerfile" "$TARGET_DIR/"
cp "$SOURCE_DIR/docker-compose.yml" "$TARGET_DIR/"
cp -r "$SOURCE_DIR/k8s" "$TARGET_DIR/"

echo "📚 Copying documentation..."
cp "$SOURCE_DIR/README.md" "$TARGET_DIR/docs/original-readme.md"
cp "$SOURCE_DIR/DEPLOYMENT.md" "$TARGET_DIR/docs/"

echo "✅ Migration completed!"
echo ""
echo "Next steps:"
echo "1. cd ../2hal9-demo"
echo "2. Update imports in copied files to use new crate structure"
echo "3. cargo check to identify and fix compilation issues"
echo "4. Run tests to ensure functionality"
echo "5. Commit the migrated code"

echo ""
echo "Note: The copied files will need manual adjustments for:"
echo "- Import paths (genius_game_server:: → genius_core::, etc.)"
echo "- Module declarations"
echo "- Dependencies between crates"