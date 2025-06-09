#!/bin/bash

# HAL9 Codebase Migration to Hierarchical Abstraction Structure
# This script reorganizes the entire codebase according to HA principles

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}HAL9 Hierarchical Abstraction Reorganization${NC}"
echo -e "${BLUE}==========================================${NC}"
echo
echo "This will reorganize the entire codebase according to cognitive levels."
echo "Each level will have its own home without forced abstraction switching."
echo

# Function to move files while preserving git history
move_with_history() {
    local src=$1
    local dst=$2
    
    if [ -e "$src" ]; then
        mkdir -p "$(dirname "$dst")"
        git mv "$src" "$dst" 2>/dev/null || mv "$src" "$dst"
        echo -e "${GREEN}✓${NC} Moved: $src → $dst"
    fi
}

# Function to create README for each level
create_level_readme() {
    local level=$1
    local title=$2
    local desc=$3
    local time=$4
    
    cat > "$level/README.md" << EOF
# $title

**Cognitive Level**: $level  
**Temporal Scope**: $time  
**Purpose**: $desc

## Overview

This level contains everything needed for ${desc,,}. All content here operates at the same level of abstraction, eliminating cognitive switching.

## Structure

$(ls -1 "$level" | grep -v README.md | awk '{print "- `" $0 "/` - "}')

## Navigation

- **Down**: Lower levels provide implementation details
- **Up**: Higher levels provide context and purpose
- **Lateral**: Everything at this level shares the same abstraction

## Principles

1. All content here shares the same temporal scope ($time)
2. No implementation details from lower levels
3. No strategic concerns from higher levels
4. Self-contained within this cognitive space
EOF
}

echo -e "${YELLOW}Phase 1: Migrating Documentation${NC}"

# L1 - Reflexive (Immediate responses)
echo "Moving L1 Reflexive content..."
move_with_history "docs/L1_operational/L1_QUICK_START.md" "L1_reflexive/status/quick-start.md"
move_with_history "docs/L1_operational/L1_MONITORING_GUIDE.md" "L1_reflexive/status/monitoring.md"
move_with_history "docs/L1_operational/L1_TROUBLESHOOTING.md" "L1_reflexive/emergency/troubleshooting.md"
move_with_history "docs/L1_operational/L1_DAILY_TASKS.md" "L1_reflexive/responses/daily-tasks.md"
move_with_history "scripts/health-check.sh" "L1_reflexive/status/scripts/health-check.sh"
move_with_history "scripts/monitor-deployment.sh" "L1_reflexive/status/scripts/monitor.sh"

# Emergency scripts
for script in test-*.sh; do
    if [ -f "$script" ]; then
        move_with_history "$script" "L1_reflexive/emergency/scripts/$script"
    fi
done

# L2 - Implementation (Code execution)
echo "Moving L2 Implementation content..."
move_with_history "hal9-core/src" "L2_implementation/neurons/core"
move_with_history "hal9-codegen/src" "L2_implementation/codegen/src"
move_with_history "docs/L2_implementation/L2_DEVELOPMENT_GUIDE.md" "L2_implementation/execution/development-guide.md"
move_with_history "docs/L2_implementation/L2_TESTING_METHODOLOGY.md" "L2_implementation/validation/testing-methodology.md"
move_with_history "benches" "L2_implementation/validation/benchmarks"
move_with_history "tests" "L2_implementation/validation/tests"

# L3 - Operational (System design)
echo "Moving L3 Operational content..."
move_with_history "hal9-server/src" "L3_operational/architecture/server"
move_with_history "hal9-browser/src" "L3_operational/architecture/browser"
move_with_history "hal9-cli/src" "L3_operational/architecture/cli"
move_with_history "k8s" "L3_operational/architecture/kubernetes"
move_with_history "docker-compose.yml" "L3_operational/configuration/docker/compose.yml"
move_with_history "Dockerfile" "L3_operational/configuration/docker/Dockerfile"
move_with_history "monitoring" "L3_operational/workflows/monitoring"
move_with_history "config" "L3_operational/configuration/system"

# L4 - Tactical (Planning)
echo "Moving L4 Tactical content..."
move_with_history "docs/L4_architecture" "L4_tactical/strategies/architecture"
move_with_history "runbooks" "L4_tactical/planning/runbooks"
move_with_history "scripts/backup-hal9.sh" "L4_tactical/planning/backup/backup.sh"
move_with_history "scripts/benchmark-trends.sh" "L4_tactical/analysis/performance/trends.sh"

# L5 - Strategic (Technical vision)
echo "Moving L5 Strategic content..."
move_with_history "docs/L5_technical_strategy" "L5_strategic/vision"
move_with_history "hal9-plugin-sdk" "L5_strategic/innovation/plugin-system"
move_with_history "examples" "L5_strategic/research/examples"

# L6 - Executive (Leadership)
echo "Moving L6 Executive content..."
move_with_history "docs/L6_executive" "L6_executive/overview/documentation"
move_with_history "CTO_DOCUMENTATION_HIERARCHY_COMPLETE.md" "L6_executive/communication/hierarchy-overview.md"
move_with_history "CTO_HIERARCHICAL_ARCHITECTURE_SUMMARY.md" "L6_executive/overview/architecture-summary.md"

# L7 - Business (Business strategy)
echo "Moving L7 Business content..."
move_with_history "docs/L7_strategic_business" "L7_business/product/documentation"
move_with_history "LICENSE" "L7_business/value/license.md"

# L8 - Visionary (Long-term)
echo "Moving L8 Visionary content..."
move_with_history "docs/L8_visionary" "L8_visionary/future/documentation"

# L9 - Universal (Timeless)
echo "Moving L9 Universal content..."
move_with_history "docs/L9_universal" "L9_universal/principles/documentation"
move_with_history "docs/L9_HIERARCHY_DEFINITION.md" "L9_universal/philosophy/hierarchy-definition.md"

echo
echo -e "${YELLOW}Phase 2: Migrating Infrastructure${NC}"

# Substrate layer
echo "Moving substrate components..."
move_with_history "hal9-core/src/substrate" "substrate/compute/runtime"
move_with_history "data" "substrate/storage/databases"
move_with_history "Cargo.toml" "substrate/tooling/rust/workspace.toml"
move_with_history "Cargo.lock" "substrate/tooling/rust/workspace.lock"

# Hidden substrate (technical necessities)
echo "Moving technical necessities to hidden substrate..."
move_with_history "target" ".substrate/build/rust"
move_with_history ".git" ".substrate/version-control/git"
for logfile in *.log; do
    if [ -f "$logfile" ]; then
        move_with_history "$logfile" ".substrate/logs/$logfile"
    fi
done

# Membrane (inter-level communication)
echo "Setting up membrane layer..."
move_with_history "hal9-core/src/hierarchical/protocol" "membrane/protocols/hierarchical"
move_with_history "hal9-core/src/hierarchical/interfaces.rs" "membrane/interfaces/core.rs"

echo
echo -e "${YELLOW}Phase 3: Creating Level Documentation${NC}"

# Create README for each level
create_level_readme "L1_reflexive" "L1 - Reflexive Layer" "Immediate operational responses and system health" "Microseconds to seconds"
create_level_readme "L2_implementation" "L2 - Implementation Layer" "Code execution and neuron implementations" "Milliseconds to seconds"
create_level_readme "L3_operational" "L3 - Operational Layer" "System design and architecture" "Seconds to minutes"
create_level_readme "L4_tactical" "L4 - Tactical Layer" "Planning and tactical strategies" "Minutes to hours"
create_level_readme "L5_strategic" "L5 - Strategic Layer" "Technical vision and innovation" "Hours to days"
create_level_readme "L6_executive" "L6 - Executive Layer" "Leadership view and communication" "Days to weeks"
create_level_readme "L7_business" "L7 - Business Layer" "Business strategy and product vision" "Weeks to months"
create_level_readme "L8_visionary" "L8 - Visionary Layer" "Long-term vision and paradigm shifts" "Months to years"
create_level_readme "L9_universal" "L9 - Universal Layer" "Timeless principles and philosophy" "Eternal"

echo
echo -e "${YELLOW}Phase 4: Creating Navigation Structure${NC}"

# Create root navigation
cat > "NAVIGATION.md" << 'EOF'
# HAL9 Hierarchical Navigation

Welcome to the cognitively-organized HAL9 codebase. Choose your cognitive level:

## 🚀 Quick Access by Role

### Operators & SREs → [L1 Reflexive](L1_reflexive/)
- System health, monitoring, emergency procedures
- Immediate responses, no deep thinking required

### Developers → [L2 Implementation](L2_implementation/)
- Code, neurons, execution engines
- Implementation details and validation

### Architects → [L3 Operational](L3_operational/)
- System design, configuration, workflows
- Architecture and optimization

### Tech Leads → [L4 Tactical](L4_tactical/)
- Planning, analysis, strategies
- Performance and adaptation

### CTOs & Principal Engineers → [L5 Strategic](L5_strategic/)
- Technical vision, innovation, research
- Long-term technical evolution

### Executives → [L6 Executive](L6_executive/)
- Overviews, decisions, metrics
- Leadership communication

### Product & Business → [L7 Business](L7_business/)
- Product strategy, market analysis
- Business value and growth

### Visionaries → [L8 Visionary](L8_visionary/)
- Future scenarios, paradigm shifts
- Moonshot projects

### Philosophers → [L9 Universal](L9_universal/)
- Core principles, universal patterns
- Timeless wisdom

## 🌐 Infrastructure Layers

### [Substrate](substrate/)
Supporting infrastructure for all cognitive levels

### [Membrane](membrane/)
Inter-level communication and protocols

## 📖 Principles

1. **Stay at your level** - Everything you need is in one place
2. **No forced switching** - Related content lives together
3. **Natural navigation** - Move up for context, down for details
4. **Self-contained** - Each level is complete unto itself
EOF

echo
echo -e "${YELLOW}Phase 5: Updating Build Configuration${NC}"

# Create new workspace Cargo.toml at substrate level
cat > "substrate/tooling/rust/Cargo.toml" << 'EOF'
[workspace]
members = [
    "../../L2_implementation/neurons/core",
    "../../L2_implementation/codegen",
    "../../L3_operational/architecture/server",
    "../../L3_operational/architecture/browser",
    "../../L3_operational/architecture/cli",
    "../../L5_strategic/innovation/plugin-system",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["HAL9 Team"]
license = "MIT"

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
EOF

# Create symlinks for backward compatibility (temporary)
ln -sf substrate/tooling/rust/Cargo.toml Cargo.toml 2>/dev/null || true
ln -sf substrate/tooling/rust/Cargo.lock Cargo.lock 2>/dev/null || true

echo
echo -e "${YELLOW}Phase 6: Cleaning Up${NC}"

# Remove empty directories
find . -type d -empty -delete 2>/dev/null || true

# Remove old docs directory if empty
rmdir docs 2>/dev/null || true

echo
echo -e "${GREEN}✅ Migration Complete!${NC}"
echo
echo "The codebase has been reorganized according to Hierarchical Abstraction principles."
echo "Each cognitive level now has its own home:"
echo
echo "  L1 → Immediate responses (operators)"
echo "  L2 → Implementation (developers)"
echo "  L3 → Design (architects)"
echo "  L4 → Planning (tech leads)"
echo "  L5 → Vision (CTOs)"
echo "  L6 → Leadership (executives)"
echo "  L7 → Business (product)"
echo "  L8 → Future (visionaries)"
echo "  L9 → Eternal (philosophers)"
echo
echo "Navigate using NAVIGATION.md at the root level."
echo
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Review the new structure"
echo "2. Update any remaining import paths"
echo "3. Test that builds still work"
echo "4. Commit the reorganization"