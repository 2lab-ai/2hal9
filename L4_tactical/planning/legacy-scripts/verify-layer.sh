#!/bin/bash

# HAL9 Layer Verification Script
# Validates individual layer functionality in the hierarchical architecture

set -e

# Configuration
LAYER=${1:-""}
NAMESPACE="hal9"
VERBOSE=${2:-""}

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Usage
if [ -z "$LAYER" ]; then
    echo "Usage: $0 <layer> [--verbose]"
    echo "Layers: substrate, protocol, l1-reflexive, l2-implementation, l3-operational, l4-tactical, l5-strategic, orchestration, intelligence"
    exit 1
fi

echo -e "${BLUE}HAL9 Layer Verification: $LAYER${NC}"
echo "================================"

# Function to run layer-specific tests
run_layer_tests() {
    local layer=$1
    local pod=$(kubectl get pod -n $NAMESPACE -l layer=$layer -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -z "$pod" ]; then
        echo -e "${RED}✗ No pods found for layer: $layer${NC}"
        exit 1
    fi
    
    case $layer in
        "substrate")
            test_substrate_layer $pod
            ;;
        "protocol")
            test_protocol_layer $pod
            ;;
        "l1-reflexive")
            test_l1_reflexive_layer $pod
            ;;
        "l2-implementation")
            test_l2_implementation_layer $pod
            ;;
        "l3-operational")
            test_l3_operational_layer $pod
            ;;
        "l4-tactical")
            test_l4_tactical_layer $pod
            ;;
        "l5-strategic")
            test_l5_strategic_layer $pod
            ;;
        "orchestration")
            test_orchestration_layer $pod
            ;;
        "intelligence")
            test_intelligence_layer $pod
            ;;
        *)
            echo -e "${RED}Unknown layer: $layer${NC}"
            exit 1
            ;;
    esac
}

# Substrate Layer Tests
test_substrate_layer() {
    local pod=$1
    echo "Testing Substrate Layer..."
    
    # Test storage functionality
    echo -n "  Storage Operations: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test storage --write --read --delete > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        [ "$VERBOSE" == "--verbose" ] && kubectl exec -n $NAMESPACE $pod -- hal9-cli test storage --write --read --delete
        return 1
    fi
    
    # Test transport functionality
    echo -n "  Transport Layer: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test transport --loopback > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test resource management
    echo -n "  Resource Management: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test resources --allocate --release > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test runtime
    echo -n "  Runtime Environment: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test runtime --health > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# Protocol Layer Tests
test_protocol_layer() {
    local pod=$1
    echo "Testing Protocol Layer..."
    
    # Test signal protocol
    echo -n "  Signal Protocol: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test protocol --signal > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test gradient protocol
    echo -n "  Gradient Protocol: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test protocol --gradient > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test consensus protocol
    echo -n "  Consensus Protocol: "
    local consensus_result=$(kubectl exec -n $NAMESPACE $pod -- hal9-cli test protocol --consensus 2>&1)
    if [[ "$consensus_result" == *"passed"* ]] || [[ "$consensus_result" == *"success"* ]]; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        [ "$VERBOSE" == "--verbose" ] && echo "$consensus_result"
        return 1
    fi
    
    # Test versioning
    echo -n "  Protocol Versioning: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test protocol --version-negotiation > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# L1 Reflexive Layer Tests
test_l1_reflexive_layer() {
    local pod=$1
    echo "Testing L1 Reflexive Layer..."
    
    # Test pattern matching
    echo -n "  Pattern Matching: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test reflexive --pattern-match > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test response time
    echo -n "  Response Time: "
    local response_time=$(kubectl exec -n $NAMESPACE $pod -- hal9-cli test reflexive --measure-latency | grep -oP 'latency: \K\d+' || echo "999")
    if [ "$response_time" -lt 10 ]; then
        echo -e "${GREEN}✓ (${response_time}ms)${NC}"
    else
        echo -e "${RED}✗ (${response_time}ms)${NC}"
        return 1
    fi
    
    # Test cache functionality
    echo -n "  Cache Operations: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test reflexive --cache > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# L2 Implementation Layer Tests
test_l2_implementation_layer() {
    local pod=$1
    echo "Testing L2 Implementation Layer..."
    
    # Test code generation
    echo -n "  Code Generation: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test implementation --codegen > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test execution engine
    echo -n "  Execution Engine: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test implementation --execute > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test optimization
    echo -n "  Code Optimization: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test implementation --optimize > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# L3 Operational Layer Tests
test_l3_operational_layer() {
    local pod=$1
    echo "Testing L3 Operational Layer..."
    
    # Test design decisions
    echo -n "  Design Decision Engine: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test operational --design > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test architecture optimization
    echo -n "  Architecture Optimization: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test operational --optimize-arch > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test resource planning
    echo -n "  Resource Planning: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test operational --resource-plan > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# L4 Tactical Layer Tests
test_l4_tactical_layer() {
    local pod=$1
    echo "Testing L4 Tactical Layer..."
    
    # Test planning engine
    echo -n "  Planning Engine: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test tactical --planning > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test strategy formulation
    echo -n "  Strategy Formulation: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test tactical --strategy > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test adaptation mechanisms
    echo -n "  Adaptation Mechanisms: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test tactical --adapt > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# L5 Strategic Layer Tests
test_l5_strategic_layer() {
    local pod=$1
    echo "Testing L5 Strategic Layer..."
    
    # Test vision processing
    echo -n "  Vision Processing: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test strategic --vision > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test goal management
    echo -n "  Goal Management: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test strategic --goals > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test learning integration
    echo -n "  Learning Integration: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test strategic --learning > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# Orchestration Layer Tests
test_orchestration_layer() {
    local pod=$1
    echo "Testing Orchestration Layer..."
    
    # Test topology management
    echo -n "  Topology Management: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test orchestration --topology > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test flow control
    echo -n "  Flow Control: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test orchestration --flow > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test coordination
    echo -n "  Layer Coordination: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test orchestration --coordinate > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test routing
    echo -n "  Signal Routing: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test orchestration --routing > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# Intelligence Layer Tests
test_intelligence_layer() {
    local pod=$1
    echo "Testing Intelligence Layer..."
    
    # Test meta-learning
    echo -n "  Meta-Learning: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test intelligence --meta-learning > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test emergence detection
    echo -n "  Emergence Detection: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test intelligence --emergence > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test self-organization
    echo -n "  Self-Organization: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test intelligence --self-organize > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test creativity
    echo -n "  Creativity Engine: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test intelligence --creativity > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# Run the tests
run_layer_tests $LAYER

# Check inter-layer communication if not substrate
if [ "$LAYER" != "substrate" ]; then
    echo
    echo "Testing Inter-Layer Communication..."
    
    pod=$(kubectl get pod -n $NAMESPACE -l layer=$LAYER -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    # Test upward communication
    echo -n "  Upward Communication: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test communication --direction up > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
    fi
    
    # Test downward communication  
    echo -n "  Downward Communication: "
    if kubectl exec -n $NAMESPACE $pod -- hal9-cli test communication --direction down > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
    fi
fi

echo
echo -e "${GREEN}Layer verification complete!${NC}"