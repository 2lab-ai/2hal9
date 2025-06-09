#!/bin/bash

# Test script for 3-neuron HAL9 demo

echo "🚀 Testing HAL9 3-neuron orchestration..."

# Function to send a signal
send_signal() {
    local content="$1"
    echo -e "\n📨 Sending signal: '$content'"
    
    curl -X POST http://localhost:8080/api/v1/signal \
        -H "Content-Type: application/json" \
        -d "{
            \"content\": \"$content\",
            \"layer\": \"L4\",
            \"neuron_id\": \"neuron-1\"
        }" \
        2>/dev/null | jq '.'
    
    # Give it time to process
    sleep 2
}

# Check server status
echo "📊 Checking server status..."
curl -s http://localhost:8080/api/v1/status | jq '.'

# Test different scenarios
echo -e "\n🧪 Testing Demo Scenarios..."

# Scenario 1: Web app creation
send_signal "Create a web application for task management"

# Scenario 2: Data analysis
send_signal "Analyze customer data and generate insights"

# Scenario 3: API design
send_signal "Design a RESTful API for user authentication"

echo -e "\n✅ Demo complete!"