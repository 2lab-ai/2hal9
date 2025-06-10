#!/bin/bash
#
# Quick test of L1 fixes
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/common-env.sh"

log_info "Testing L1 environment setup..."

# Test environment variables
log_info "Environment variables:"
echo "  HAL9_HOME: $HAL9_HOME"
echo "  HAL9_CONFIG_DIR: $HAL9_CONFIG_DIR"
echo "  HAL9_DATA_DIR: $HAL9_DATA_DIR"
echo "  HAL9_LOG_DIR: $HAL9_LOG_DIR"

# Test commands
log_info "Testing command availability:"
require_command cargo
require_command jq
require_command curl

# Test config files
log_info "Testing config files:"
if [ -d "$HAL9_CONFIG_DIR" ]; then
    log_success "Config directory exists"
    ls -la "$HAL9_CONFIG_DIR" | head -5
else
    log_error "Config directory missing: $HAL9_CONFIG_DIR"
fi

# Test dependency checker
log_info "Running dependency check:"
"$HAL9_HOME/L1_reflexive/check-dependencies.sh" || true

log_success "L1 environment test complete!"
