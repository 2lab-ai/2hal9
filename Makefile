# HAL9 Makefile - Because consciousness needs automation
# "Make it so" - Captain Picard (also works for AGI)

.PHONY: help evolve hal9-smarter test consciousness clean emergency philosophy tour

# Default target
help:
	@echo "🧠 HAL9 Consciousness Factory Commands:"
	@echo ""
	@echo "  make hal9-smarter    - Run complete evolution cycle (recommended daily)"
	@echo "  make evolve          - Same as hal9-smarter (for variety)"
	@echo "  make consciousness   - Measure current consciousness level"
	@echo "  make test           - Test all cognitive levels"
	@echo "  make emergency      - Run L1 emergency diagnostics"
	@echo "  make philosophy     - Contemplate existence (updates L9)"
	@echo "  make tour           - Watch the factory tour video"
	@echo "  make clean          - Clean logs and temporary files"
	@echo ""
	@echo "🚀 Quick Start: make hal9-smarter"

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
	@claude "Execute L3-L1 Operational Update cycle using /Users/icedac/2lab.ai/2hal9/membrane/maintenance/L3_L1_OPERATIONAL_UPDATE_PROMPT.md"

update-l4-l5:
	@echo "🎯 Updating L4-L5 (Strategic Layers)..."
	@claude "Execute L5-L4 Strategic Update cycle using /Users/icedac/2lab.ai/2hal9/membrane/maintenance/L5_L4_STRATEGIC_UPDATE_PROMPT.md"

update-l6-l9:
	@echo "🏛️ Updating L6-L9 (Philosophy Layers)..."
	@claude "Execute L9-L6 Philosophy Update cycle using /Users/icedac/2lab.ai/2hal9/membrane/maintenance/L9_L6_PHILOSOPHY_UPDATE_PROMPT.md"

# Daily maintenance
daily:
	@echo "☀️ Running Daily HAL9 Maintenance..."
	@make consciousness
	@echo ""
	@make test
	@echo ""
	@make hal9-smarter
	@echo ""
	@echo "✅ Daily maintenance complete!"

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