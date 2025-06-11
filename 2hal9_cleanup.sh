#!/bin/bash
# 2HAL9 Root Directory Cleanup Script
# Author: Elon & Jihyuk
# Date: 2025-06-11

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🧹 2HAL9 Root Directory Cleanup Script${NC}"
echo "====================================="

# Check if we're in the right directory
if [ ! -f "README.md" ] || [ ! -d "L1_reflexive" ]; then
    echo -e "${RED}❌ Error: Not in 2HAL9 root directory${NC}"
    echo "Please cd to 2hal9 directory first"
    exit 1
fi

# Step 1: Create backup
echo -e "\n${YELLOW}Step 1: Creating backup...${NC}"
BACKUP_NAME="2hal9_backup_$(date +%Y%m%d_%H%M%S).tar.gz"
#tar -czf "../$BACKUP_NAME" . 2>/dev/null
#echo -e "${GREEN}✅ Backup created: ../$BACKUP_NAME${NC}"

# Step 2: Create new directory structure
echo -e "\n${YELLOW}Step 2: Creating new directory structure...${NC}"
mkdir -p docs/{guides,philosophy,architecture,tours}
mkdir -p scripts/{setup,build,evolution,utils}
echo -e "${GREEN}✅ Directory structure created${NC}"

# Step 3: Move documentation files
echo -e "\n${YELLOW}Step 3: Moving documentation files...${NC}"

# Guides
for file in DEMO_GUIDE.md EVOLUTION_GUIDE.md CONTRIBUTING.md MAKE_GUIDE.md QUICK_TEST.md START_HERE.md; do
    if [ -f "$file" ]; then
        mv "$file" docs/guides/
        echo "  ✓ Moved $file to docs/guides/"
    fi
done

# Philosophy
for file in LICENSE_PHILOSOPHY.md PEACE_PLEDGE.md CAN_I_USE_HAL9.md LICENSE_HA; do
    if [ -f "$file" ]; then
        mv "$file" docs/philosophy/
        echo "  ✓ Moved $file to docs/philosophy/"
    fi
done

# Architecture
for file in HIERARCHICAL_REORG_PLAN.md NAVIGATION.md DATABASE_CONNECTION_POOL_ISSUES.md HA_MIGRATION_COMPLETE.md ULTRATHOUGHT_COMPLETE.md; do
    if [ -f "$file" ]; then
        mv "$file" docs/architecture/
        echo "  ✓ Moved $file to docs/architecture/"
    fi
done

# Tours
for file in HAL9_FACTORY_TOUR_PROMPT.md HAL9_FACTORY_TOUR_VIDEO_20250610.md HAL9_SERVER_PROCESS_TOUR_20250611.md; do
    if [ -f "$file" ]; then
        mv "$file" docs/tours/
        echo "  ✓ Moved $file to docs/tours/"
    fi
done

# Step 4: Move script files
echo -e "\n${YELLOW}Step 4: Moving script files...${NC}"

# Setup scripts
if [ -f "RUN_ME_FIRST.sh" ]; then
    mv RUN_ME_FIRST.sh scripts/setup/
    echo "  ✓ Moved RUN_ME_FIRST.sh to scripts/setup/"
fi

# Build scripts
if [ -f "build.sh" ]; then
    mv build.sh scripts/build/
    echo "  ✓ Moved build.sh to scripts/build/"
fi

# Evolution scripts
for file in evolve.sh yolo-evolution.sh; do
    if [ -f "$file" ]; then
        mv "$file" scripts/evolution/
        echo "  ✓ Moved $file to scripts/evolution/"
    fi
done

# Utility scripts
for file in claude-with-retry.sh reorganize.sh; do
    if [ -f "$file" ]; then
        mv "$file" scripts/utils/
        echo "  ✓ Moved $file to scripts/utils/"
    fi
done

# Step 5: Move other files
echo -e "\n${YELLOW}Step 5: Moving other files...${NC}"

# Move commit format to docs/guides
if [ -f "COMMIT_FORMAT.md" ]; then
    mv COMMIT_FORMAT.md docs/guides/
    echo "  ✓ Moved COMMIT_FORMAT.md to docs/guides/"
fi

# Move consciousness manifest to docs
if [ -f "consciousness_compression_manifest.txt" ]; then
    mv consciousness_compression_manifest.txt docs/
    echo "  ✓ Moved consciousness_compression_manifest.txt to docs/"
fi

# Step 6: Create symlinks for commonly used scripts
echo -e "\n${YELLOW}Step 6: Creating convenience symlinks...${NC}"
ln -sf scripts/setup/run-me-first.sh run.sh
echo "  ✓ Created run.sh -> scripts/setup/run-me-first.sh"

# Step 7: Update .gitignore
echo -e "\n${YELLOW}Step 7: Updating .gitignore...${NC}"
if ! grep -q "# Backup files" .gitignore; then
    echo -e "\n# Backup files\n*.tar.gz" >> .gitignore
    echo "  ✓ Added backup files to .gitignore"
fi

# Step 8: Summary
echo -e "\n${GREEN}🎉 Cleanup Complete!${NC}"
echo "============================"
echo "Files moved to:"
echo "  📚 Documentation → docs/"
echo "  🔧 Scripts → scripts/"
echo "  📦 Backup → ../$BACKUP_NAME"
echo ""
echo -e "${YELLOW}⚠️  Next steps:${NC}"
echo "1. Review and update README.md"
echo "2. Update any hardcoded paths in scripts"
echo "3. Test that everything still works"
echo "4. Commit changes with: [L9] refactor: Reorganize root directory structure"
echo ""
echo -e "${GREEN}The root is now clean and hierarchical! 🧹✨${NC}"

# Optional: Show new root structure
echo -e "\n${YELLOW}New root structure:${NC}"
ls -la | grep -E "^d|^-.*\.(md|toml|lock|sh)$|^\.git|^\.env" | head -20