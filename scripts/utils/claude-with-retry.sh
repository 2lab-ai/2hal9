#!/bin/bash
# Claude command wrapper with automatic rate limit handling
# Used by Makefile for all claude commands

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get the claude command and prompt
CLAUDE_CMD="$1"
shift
PROMPT="$@"

# Function to extract wait time from rate limit error
extract_wait_time() {
    local error_msg="$1"
    
    # Common patterns in rate limit errors:
    # "Please try again at 14:30"
    # "Rate limit exceeded. Try again in 5 minutes"
    # "429 Too Many Requests - wait 60 seconds"
    
    # Try HH:MM format
    if echo "$error_msg" | grep -oE '[0-9]{1,2}:[0-9]{2}' > /dev/null; then
        echo "$error_msg" | grep -oE '[0-9]{1,2}:[0-9]{2}' | head -1
        return 0
    fi
    
    # Try "X minutes"
    if echo "$error_msg" | grep -oE '[0-9]+ minutes?' > /dev/null; then
        local minutes=$(echo "$error_msg" | grep -oE '[0-9]+ minutes?' | grep -oE '[0-9]+' | head -1)
        echo "$minutes"
        return 0
    fi
    
    # Try "X seconds"
    if echo "$error_msg" | grep -oE '[0-9]+ seconds?' > /dev/null; then
        local seconds=$(echo "$error_msg" | grep -oE '[0-9]+ seconds?' | grep -oE '[0-9]+' | head -1)
        echo "$((seconds / 60 + 1))"  # Convert to minutes, round up
        return 0
    fi
    
    # Default: wait 2 minutes
    echo "2"
}

# Main execution with retry
attempt=0
max_attempts=3

while [ $attempt -lt $max_attempts ]; do
    # Try to run the command
    if output=$($CLAUDE_CMD "$PROMPT" 2>&1); then
        # Success!
        echo "$output"
        exit 0
    else
        # Check if it's a rate limit error
        if echo "$output" | grep -iE "rate limit|429|too many requests|try again" > /dev/null; then
            echo -e "${YELLOW}⚠️  Rate limit detected${NC}" >&2
            
            # Extract wait time
            wait_minutes=$(extract_wait_time "$output")
            wait_seconds=$((wait_minutes * 60))
            
            echo -e "${YELLOW}⏳ Waiting ${wait_minutes} minutes before retry...${NC}" >&2
            
            # Show countdown
            while [ $wait_seconds -gt 0 ]; do
                printf "\r${YELLOW}Time remaining: %02d:%02d${NC}" $((wait_seconds/60)) $((wait_seconds%60)) >&2
                sleep 1
                ((wait_seconds--))
            done
            printf "\r${GREEN}✓ Retrying...                    ${NC}\n" >&2
            
            ((attempt++))
            continue
        else
            # Other error - pass it through
            echo "$output" >&2
            exit 1
        fi
    fi
done

echo -e "${RED}❌ Max retry attempts reached${NC}" >&2
exit 1