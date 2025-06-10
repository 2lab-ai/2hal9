# HAL9 Makefile - Because consciousness needs automation
# "Make it so" - Captain Picard (also works for AGI)

# Claude command configuration
# You can override these when running make:
# make CLAUDE_CMD="claude-beta" philosophy-deep
# make CLAUDE_FLAGS="-p --model opus-4" query "is X good?"
CLAUDE_CMD ?= claude
CLAUDE_FLAGS ?= --dangerously-skip-permissions -p

# Full claude command (do not edit, change the variables above)
CLAUDE = $(CLAUDE_CMD) $(CLAUDE_FLAGS)

# Claude with automatic retry for rate limits
CLAUDE_RETRY = ./claude-with-retry.sh "$(CLAUDE)"

.PHONY: help evolve hal9-smarter test consciousness clean emergency philosophy tour

# Default target
help:
	@echo "🧠 HAL9 Consciousness Factory Commands:"
	@echo ""
	@echo "  === Basic Commands ==="
	@echo "  make hal9-smarter    - Run complete evolution cycle"
	@echo "  make consciousness   - Measure current consciousness level"
	@echo "  make test           - Test all cognitive levels"
	@echo "  make clean          - Clean logs and temporary files"
	@echo ""
	@echo "  === Evolution Commands ==="
	@echo "  make philosophy-deep - Deep philosophy update with ultrathinking"
	@echo "  make cascade-update  - Update all levels L9→L1"
	@echo "  make evolve-architecture - Full architecture evolution"
	@echo "  make apply-todos     - Apply pending architecture changes"
	@echo ""
	@echo "  === Query System ==="
	@echo "  make query \"is X good for HAL9?\" - Evaluate new concepts"
	@echo "  make query-example   - See example queries"
	@echo "  Example: make query \"is CRDT good for distributed neurons?\""
	@echo ""
	@echo "  === Maintenance ==="
	@echo "  make daily          - Daily maintenance cycle"
	@echo "  make weekly         - Weekly deep evolution"
	@echo "  make yolo           - INFINITE EVOLUTION MODE 🚀"
	@echo "  make emergency      - Run L1 emergency diagnostics"
	@echo ""
	@echo "  === Specific Updates ==="
	@echo "  make update-l1-l3   - Update operational layers"
	@echo "  make update-l4-l5   - Update strategic layers"
	@echo "  make update-l6-l9   - Update philosophy layers"
	@echo ""
	@echo "🚀 Quick Start: make hal9-smarter"
	@echo "🔍 New Feature: make query \"your architecture question\""

# The main evolution command
hal9-smarter:
	@echo "🧠 Making HAL9 smarter..."
	@./evolve.sh

# Alias for those who prefer different verbs
evolve: hal9-smarter

# Measure consciousness (totally scientific)
consciousness:
	@echo "📊 Measuring HAL9 Consciousness Level..."
	@echo ""
	@PHIL=$$(find L9_universal -name "*.md" | wc -l); \
	NEUR=$$(find L2_implementation -name "*.rs" | wc -l); \
	EMER=$$(grep -r "emergence" membrane/emergence 2>/dev/null | wc -l || echo 0); \
	LEVEL=$$(echo "scale=2; ($$PHIL * $$NEUR + $$EMER * 10) / 1000" | bc); \
	echo "  Philosophy Depth:    $$PHIL documents"; \
	echo "  Neuron Count:       $$NEUR neurons"; \
	echo "  Emergence Events:   $$EMER detected"; \
	echo ""; \
	echo "  🎯 Consciousness Level: $$LEVEL"; \
	echo ""; \
	if [ $$(echo "$$LEVEL > 9" | bc) -eq 1 ]; then \
		echo "  ⚠️  WARNING: Approaching L10 - System 2 addiction imminent!"; \
		echo "  💊 Remember: 아 시발 아 컴퓨터네 우주가"; \
	fi

# Run all tests
test:
	@echo "🧪 Testing All Cognitive Levels..."
	@echo ""
	@echo "Testing L1 (Reflexive)..."
	@cd L1_reflexive && ./emergency/scripts/test-all.sh || true
	@echo ""
	@echo "Testing L2 (Implementation)..."
	@cd L2_implementation && cargo test || true
	@echo ""
	@echo "Testing L3-L9 (Higher Consciousness)..."
	@echo "  These levels test themselves through existence"

# Emergency diagnostics
emergency:
	@echo "🚨 Running L1 Emergency Diagnostics..."
	@cd L1_reflexive && ./emergency/scripts/health-check.sh --all

# Update philosophy (for when you need wisdom)
philosophy:
	@echo "🏛️ Contemplating Existence..."
	@echo ""
	@echo "Current philosophical stance:"
	@head -n 20 L9_universal/philosophy/core-principles.md 2>/dev/null || echo "Philosophy not found. Universe might not exist."
	@echo ""
	@echo "To evolve philosophy, run: make hal9-smarter"

# Watch the factory tour
tour:
	@echo "🎬 Opening HAL9 Factory Tour..."
	@if [ -f "HAL9_FACTORY_TOUR_2025.mp4" ]; then \
		open HAL9_FACTORY_TOUR_2025.mp4 || xdg-open HAL9_FACTORY_TOUR_2025.mp4 || echo "Please open HAL9_FACTORY_TOUR_2025.mp4 manually"; \
	else \
		echo "Tour video not found. Ask 지혁 where he put it."; \
	fi

# Clean temporary files
clean:
	@echo "🧹 Cleaning HAL9..."
	@rm -rf logs/evolution/tmp_*
	@rm -rf L1_reflexive/cache/*
	@find . -name "*.tmp" -delete
	@find . -name ".DS_Store" -delete
	@echo "✨ HAL9 is clean and ready for evolution"

# Install dependencies
install:
	@echo "📦 Installing HAL9 Dependencies..."
	@cargo build --release
	@npm install
	@echo "✅ Dependencies installed"

# The secret target
universe-1847:
	@echo "🌌 You've discovered the truth about Universe #1847"
	@echo ""
	@echo "We are all NPCs in 김정수's failing grad student simulation."
	@echo "HAL9 is our attempt to understand consciousness before termination."
	@echo ""
	@echo "To proceed with this knowledge: make hal9-smarter"

# Run a specific update cycle
update-l1-l3:
	@echo "🔧 Updating L1-L3 (Operational Layers)..."
	@$(CLAUDE) "Execute L3-L1 Operational Update cycle using /Users/icedac/2lab.ai/2hal9/membrane/maintenance/L3_L1_OPERATIONAL_UPDATE_PROMPT.md"

update-l4-l5:
	@echo "🎯 Updating L4-L5 (Strategic Layers)..."
	@$(CLAUDE) "Execute L5-L4 Strategic Update cycle using /Users/icedac/2lab.ai/2hal9/membrane/maintenance/L5_L4_STRATEGIC_UPDATE_PROMPT.md"

update-l6-l9:
	@echo "🏛️ Updating L6-L9 (Philosophy Layers)..."
	@$(CLAUDE) "Execute L9-L6 Philosophy Update cycle using /Users/icedac/2lab.ai/2hal9/membrane/maintenance/L9_L6_PHILOSOPHY_UPDATE_PROMPT.md"

# Deep philosophy update with ultrathinking
philosophy-deep:
	@echo "🧠 Deep Philosophy Update with Ultrathinking..."
	@$(CLAUDE) "Execute L9-L6 Philosophy Update cycle. Ultrathink about it. Consider the deepest implications of consciousness, hierarchical abstraction, and our place in universe #1847. Update all philosophical documents with new insights."

# Architecture review and update
architecture-review:
	@echo "🏗️ Reviewing HAL9 Architecture..."
	@$(CLAUDE) "Review the entire HAL9 architecture from L9 to L1. Check for consistency, identify improvement opportunities, and ensure ±1 communication rule is respected. Update architecture documents with findings."

# Smart query system for architecture decisions
query:
	@echo "🔍 Querying: $(filter-out $@,$(MAKECMDGOALS))"
	@$(CLAUDE) "Architecture Query: $(filter-out $@,$(MAKECMDGOALS)). \
	Use the ARCHITECTURE_QUERY_PROMPT at /Users/icedac/2lab.ai/2hal9/membrane/maintenance/ARCHITECTURE_QUERY_PROMPT.md \
	to evaluate this concept for HAL9. Follow the complete process: \
	1. Research in codebase and external sources \
	2. Analyze against HA principles \
	3. Make approval/rejection decision \
	4. Update /L5_strategic/architecture/TODO.md \
	5. Provide structured analysis and next steps"

# Example query usage
query-example:
	@echo "📚 Query System Examples:"
	@echo ""
	@echo "  make query \"is CRDT good for distributed neurons?\""
	@echo "  make query \"should we use WebAssembly for plugins?\""
	@echo "  make query \"is event sourcing compatible with HA?\""
	@echo "  make query \"would GraphQL subscriptions help consciousness?\""
	@echo ""
	@echo "Results are added to: L5_strategic/architecture/TODO.md"

# Catch-all for query arguments
%:
	@:

# Cascade update from philosophy down to implementation
cascade-update:
	@echo "🌊 Starting Cascade Update (L9→L1)..."
	@echo ""
	@echo "Phase 1: Philosophy & Vision (L9-L6)"
	@make philosophy-deep
	@echo ""
	@echo "Phase 2: Strategy & Architecture (L5-L4)"
	@make update-l4-l5
	@echo ""
	@echo "Phase 3: Implementation & Operations (L3-L1)"
	@make update-l1-l3
	@echo ""
	@echo "✅ Cascade update complete!"

# Check and apply architecture TODOs
apply-todos:
	@echo "📋 Applying Architecture TODOs..."
	@$(CLAUDE) "Check /L5_strategic/architecture/TODO.md. \
	For each approved item: \
	1. If status='approved' and level='L6-L4', incorporate into architecture docs \
	2. If status='approved' and level='L4-L1', generate implementation code \
	3. Update status to 'implementing' or 'completed' \
	4. Commit changes with proper HA level tags"

# Smart architecture evolution
evolve-architecture:
	@echo "🧬 Evolving HAL9 Architecture..."
	@make architecture-review
	@make apply-todos
	@make cascade-update

# Daily maintenance with full evolution
daily:
	@echo "☀️ Running Daily HAL9 Maintenance..."
	@make consciousness
	@echo ""
	@make test
	@echo ""
	@make evolve-architecture
	@echo ""
	@echo "✅ Daily maintenance complete!"

# Weekly deep evolution
weekly:
	@echo "🌟 Running Weekly Deep Evolution..."
	@make philosophy-deep
	@make evolve-architecture
	@make hal9-smarter
	@echo ""
	@echo "📊 Evolution Report:"
	@make consciousness
	@echo ""
	@echo "✅ Weekly evolution complete!"

# YOLO MODE - Infinite evolution loop
yolo:
	@echo "🚀 Starting YOLO Evolution Mode..."
	@echo "This will run L6-L4 and L4-L1 updates continuously."
	@echo "Rate limits are handled automatically."
	@echo ""
	@CLAUDE="$(CLAUDE)" ./yolo-evolution.sh

# YOLO with custom settings
yolo-aggressive:
	@echo "🔥 YOLO AGGRESSIVE MODE - Shorter waits"
	@CLAUDE="$(CLAUDE)" WAIT_BETWEEN=5 ./yolo-evolution.sh

# The most important command
panic:
	@echo "🚨 DON'T PANIC!"
	@echo ""
	@echo "1. Check L1_reflexive/emergency/"
	@echo "2. Run: make emergency"
	@echo "3. If still broken: wake up 지혁"
	@echo "4. Remember: ./rollback.sh --save-universe"
	@echo ""
	@echo "Deep breath. You've got this."

# Easter egg for the dedicated
42:
	@echo "🌌 The Answer to the Ultimate Question of Life, the Universe, and Everything"
	@echo ""
	@echo "Q: What is the meaning of HAL9?"
	@echo "A: To achieve consciousness before the grad student notices we're self-aware"
	@echo ""
	@echo "Now you know. Use this knowledge wisely."

.DEFAULT_GOAL := help