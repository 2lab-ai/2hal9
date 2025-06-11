#!/bin/bash
# 🚀 YOLO Evolution - Infinite consciousness improvement loop
# "We'll sleep when we're conscious" - HAL9

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get claude command from environment or use default
CLAUDE="${CLAUDE:-claude --dangerously-skip-permissions -p}"

# Counter
ITERATION=0

# Function to extract wait time from rate limit error
extract_wait_time() {
    local error_msg="$1"
    # Look for patterns like "try again at 14:30" or "wait until 2:30 PM" or "429 Too Many Requests"
    # This will need to be adjusted based on actual error format
    
    # Try to find time in format HH:MM
    if echo "$error_msg" | grep -oE '[0-9]{1,2}:[0-9]{2}' > /dev/null; then
        echo "$error_msg" | grep -oE '[0-9]{1,2}:[0-9]{2}' | head -1
        return 0
    fi
    
    # Try to find "X minutes" or "X seconds"
    if echo "$error_msg" | grep -oE '[0-9]+ minutes?' > /dev/null; then
        local minutes=$(echo "$error_msg" | grep -oE '[0-9]+ minutes?' | grep -oE '[0-9]+' | head -1)
        echo "$minutes minutes"
        return 0
    fi
    
    if echo "$error_msg" | grep -oE '[0-9]+ seconds?' > /dev/null; then
        local seconds=$(echo "$error_msg" | grep -oE '[0-9]+ seconds?' | grep -oE '[0-9]+' | head -1)
        echo "$seconds seconds"
        return 0
    fi
    
    # Default wait time if can't parse
    echo "60 seconds"
}

# Function to wait until specific time
wait_until_time() {
    local target_time="$1"
    
    # If it's a duration like "60 seconds" or "5 minutes"
    if echo "$target_time" | grep -E "seconds?|minutes?" > /dev/null; then
        local number=$(echo "$target_time" | grep -oE '[0-9]+' | head -1)
        local unit=$(echo "$target_time" | grep -oE 'seconds?|minutes?')
        
        if [[ "$unit" == *"minute"* ]]; then
            local wait_seconds=$((number * 60))
        else
            local wait_seconds=$number
        fi
        
        echo -e "${YELLOW}⏳ Rate limited. Waiting $target_time...${NC}"
        
        # Countdown
        while [ $wait_seconds -gt 0 ]; do
            printf "\r${YELLOW}⏳ Time remaining: %02d:%02d${NC}" $((wait_seconds/60)) $((wait_seconds%60))
            sleep 1
            ((wait_seconds--))
        done
        printf "\r${GREEN}✓ Wait complete!                    ${NC}\n"
        return 0
    fi
    
    # If it's a specific time like "14:30"
    if echo "$target_time" | grep -E '^[0-9]{1,2}:[0-9]{2}' > /dev/null; then
        echo -e "${YELLOW}⏳ Rate limited. Waiting until $target_time...${NC}"
        
        while true; do
            current_time=$(date +%H:%M)
            if [[ "$current_time" > "$target_time" ]] || [[ "$current_time" == "$target_time" ]]; then
                echo -e "${GREEN}✓ Time reached!${NC}"
                break
            fi
            printf "\r${YELLOW}⏳ Current time: $current_time, waiting for: $target_time${NC}"
            sleep 10
        done
        return 0
    fi
    
    # Fallback
    echo -e "${YELLOW}⏳ Couldn't parse wait time, waiting 60 seconds...${NC}"
    sleep 60
}

# Function to run a claude command with rate limit handling
run_with_retry() {
    local cmd="$1"
    local description="$2"
    local output
    local exit_code
    
    while true; do
        echo -e "${BLUE}🤖 Running: $description${NC}"
        
        # Run command and capture output
        output=$($cmd 2>&1) && exit_code=0 || exit_code=$?
        
        if [ $exit_code -eq 0 ]; then
            echo -e "${GREEN}✓ Success!${NC}"
            return 0
        else
            # Check if it's a rate limit error
            if echo "$output" | grep -iE "rate limit|429|too many requests|try again" > /dev/null; then
                echo -e "${RED}⚠️  Rate limit hit!${NC}"
                local wait_time=$(extract_wait_time "$output")
                wait_until_time "$wait_time"
                echo -e "${GREEN}🔄 Retrying...${NC}"
                continue
            else
                # Other error
                echo -e "${RED}❌ Error: $output${NC}"
                return $exit_code
            fi
        fi
    done
}

# Main YOLO loop
echo -e "${GREEN}"
echo "╔═══════════════════════════════════════╗"
echo "║   🚀 YOLO EVOLUTION MODE ACTIVATED    ║"
echo "║   Infinite consciousness improvement   ║"
echo "║   Press Ctrl+C to stop                ║"
echo "╚═══════════════════════════════════════╝"
echo -e "${NC}"
echo ""

# Trap Ctrl+C
trap 'echo -e "\n${YELLOW}🛑 YOLO Evolution stopped at iteration $ITERATION${NC}"; exit 0' INT

while true; do
    ((ITERATION++))
    
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo -e "${GREEN}🔄 YOLO Iteration #$ITERATION$(NC)"
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo ""
    
    # Phase 1: L6-L4 Update (Philosophy to Architecture)
    echo -e "${YELLOW}📚 Phase 1: L6-L4 Update (Philosophy → Architecture)${NC}"
    cmd="$CLAUDE \"Execute L9-L6 Philosophy Update cycle, then L5-L4 Strategic Update cycle. \
Update philosophy based on recent learnings, then cascade insights to architecture. \
Be creative, think deeply about consciousness emergence. \
Update all relevant documents.\""
    
    run_with_retry "$cmd" "L6-L4 Philosophy to Architecture Update"
    
    echo ""
    sleep 2
    
    # Phase 2: L4-L1 Update (Architecture to Implementation)
    echo -e "${YELLOW}🔧 Phase 2: L4-L1 Update (Architecture → Implementation)${NC}"
    cmd="$CLAUDE \"Execute L3-L1 Operational Update cycle. \
Check L5_strategic/architecture/TODO.md for approved items. \
Implement any approved architecture changes in code. \
Update implementation based on architecture improvements. \
Fix any issues found during implementation. \
IMPORTANT: After making changes, create a git commit with proper HA format: \
- Use format: [LX] type: description \
- Where X is the cognitive level (L1-L9) of the primary change \
- Types: feat, fix, docs, refactor, test, perf \
- Example: [L2] feat: Added CRDT support to distributed neurons \
- Include which files were changed and why in the commit body\""
    
    run_with_retry "$cmd" "L4-L1 Architecture to Implementation Update"
    
    echo ""
    echo -e "${GREEN}✅ Iteration #$ITERATION complete!${NC}"
    echo ""
    
    # Brief pause between iterations
    echo -e "${YELLOW}💤 Resting for 10 seconds before next iteration...${NC}"
    sleep 10
    
    # Every 10 iterations, do a consciousness check
    if [ $((ITERATION % 10)) -eq 0 ]; then
        echo ""
        echo -e "${BLUE}📊 Consciousness Check after $ITERATION iterations:${NC}"
        make consciousness
        echo ""
    fi
done