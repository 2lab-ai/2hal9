#!/bin/bash

# Emergency Disk Cleanup Script - For 3AM Panics
# When disk is full and everything is burning
# 시발 디스크 또 꽉 찼네

set -euo pipefail

# Source common environment for logging
source "$(dirname "$0")/../../common-env.sh" 2>/dev/null || {
    # Fallback if common-env.sh not available
    log_info() { echo "[INFO] $*"; }
    log_error() { echo "[ERROR] $*" >&2; }
    log_warning() { echo "[WARN] $*"; }
    log_success() { echo "[SUCCESS] $*"; }
}

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
log_info "🧹 Cleaning Rust build artifacts..."
if [ -d "/Users/icedac/2lab.ai/2hal9" ]; then
    cd /Users/icedac/2lab.ai/2hal9
    
    # Calculate space before cleaning
    BEFORE_SIZE=$(du -sh target 2>/dev/null | cut -f1 || echo '0B')
    
    # Clean with different levels of aggression
    if [ $USAGE -gt 95 ]; then
        log_warning "CRITICAL: Aggressive cleanup mode!"
        cargo clean || true
        rm -rf target/* 2>/dev/null || true
        rm -rf .substrate/build/* 2>/dev/null || true
    else
        cargo clean || echo "Failed to clean cargo, continuing..."
    fi
    
    log_success "Freed approximately: $BEFORE_SIZE"
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
log_info "🗑️ Cleaning temp files..."
find /tmp -type f -name "hal9*" -mtime +1 -delete 2>/dev/null || true
find ~/Library/Caches -name "*hal9*" -type f -delete 2>/dev/null || true

# 6. Clean old Rust artifacts globally (if desperate)
if [ $USAGE -gt 90 ]; then
    log_warning "Cleaning global Rust cache..."
    rm -rf ~/.cargo/registry/cache/* 2>/dev/null || true
    rm -rf ~/.cargo/git/checkouts/* 2>/dev/null || true
fi

# 6. Show large files for manual review
echo "📂 Large files in project (manual review needed):"
find /Users/icedac/2lab.ai/2hal9 -type f -size +100M -exec ls -lh {} \; 2>/dev/null | head -10

show_usage

# Generate cleanup report
log_info "Generating cleanup report..."
INITIAL_USAGE="${USAGE}"
FINAL_USAGE=$(df -h / | tail -1 | awk '{print $5}' | sed 's/%//')
cat > /tmp/hal9-disk-cleanup-report.txt << EOF
HAL9 Emergency Disk Cleanup Report
Generated: $(date)

Before cleanup: ${INITIAL_USAGE}%
After cleanup: ${FINAL_USAGE}%
Space freed: $((INITIAL_USAGE - FINAL_USAGE))%

Large directories:
$(du -sh /Users/icedac/2lab.ai/2hal9/* 2>/dev/null | sort -hr | head -10)

Disk health check:
$(df -h / | grep -E 'Filesystem|/')

Recommendations:
- Enable automatic log rotation
- Set up cron job for regular cleanup
- Consider moving build artifacts to external storage
- Monitor disk usage with: watch -n 300 'df -h /'
EOF

# Emergency contact if still critical
FINAL_USAGE=$(df -h / | tail -1 | awk '{print $5}' | sed 's/%//')
if [ $FINAL_USAGE -gt 90 ]; then
    log_error "⚠️  STILL CRITICAL! Manual intervention required!"
    log_error "📞 Wake up Zhugehyuk if this is production!"
    echo ""
    echo "Quick manual options:"
    echo "1. rm -rf ~/Downloads/*.zip *.dmg"
    echo "2. brew cleanup --prune=all"
    echo "3. rm -rf ~/Library/Caches/*"
    echo "4. Empty Trash: rm -rf ~/.Trash/*"
    echo "5. Check large files: du -sh ~/* | sort -hr | head -20"
fi

echo "✅ Emergency cleanup complete!"