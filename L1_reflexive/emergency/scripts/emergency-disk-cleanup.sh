#!/bin/bash

# Emergency Disk Cleanup Script - For 3AM Panics
# When disk is full and everything is burning
# 시발 디스크 또 꽉 찼네

set -e

echo "🚨 EMERGENCY DISK CLEANUP INITIATED 🚨"
echo "Current disk usage:"
df -h /

# Function to show disk usage
show_usage() {
    USAGE=$(df -h / | tail -1 | awk '{print $5}' | sed 's/%//')
    echo "📊 Disk usage: ${USAGE}%"
    if [ $USAGE -lt 80 ]; then
        echo "✅ Disk usage is now safe (below 80%)"
    elif [ $USAGE -lt 90 ]; then
        echo "⚠️  Disk usage is high but manageable"
    else
        echo "🔥 CRITICAL: Disk usage is still critical!"
    fi
}

# 1. Clean Rust build artifacts (usually the biggest culprit)
echo "🧹 Cleaning Rust build artifacts..."
if [ -d "/Users/icedac/2lab.ai/2hal9" ]; then
    cd /Users/icedac/2lab.ai/2hal9
    cargo clean || echo "Failed to clean cargo, continuing..."
    echo "Freed: $(du -sh target 2>/dev/null | cut -f1 || echo '0B')"
fi

# 2. Clean old log files (older than 3 days)
echo "📝 Cleaning old log files..."
find /Users/icedac/2lab.ai/2hal9 -name "*.log" -type f -mtime +3 -delete 2>/dev/null || true
find /tmp -name "hal9*.log" -type f -mtime +1 -delete 2>/dev/null || true

# 3. Clean npm cache if exists
echo "📦 Cleaning npm cache..."
npm cache clean --force 2>/dev/null || true

# 4. Clean Docker if running (optional - only if desperate)
if command -v docker &> /dev/null; then
    echo "🐳 Cleaning Docker..."
    docker system prune -f 2>/dev/null || true
fi

# 5. Clean system temp files
echo "🗑️ Cleaning temp files..."
find /tmp -type f -name "hal9*" -mtime +1 -delete 2>/dev/null || true
find ~/Library/Caches -name "*hal9*" -type f -delete 2>/dev/null || true

# 6. Show large files for manual review
echo "📂 Large files in project (manual review needed):"
find /Users/icedac/2lab.ai/2hal9 -type f -size +100M -exec ls -lh {} \; 2>/dev/null | head -10

show_usage

# Emergency contact if still critical
FINAL_USAGE=$(df -h / | tail -1 | awk '{print $5}' | sed 's/%//')
if [ $FINAL_USAGE -gt 90 ]; then
    echo "⚠️  STILL CRITICAL! Manual intervention required!"
    echo "📞 Wake up Zhugehyuk if this is production!"
    echo ""
    echo "Quick manual options:"
    echo "1. rm -rf ~/Downloads/*.zip *.dmg"
    echo "2. brew cleanup"
    echo "3. Check ~/Library/Caches"
    echo "4. Empty Trash!"
fi

echo "✅ Emergency cleanup complete!"