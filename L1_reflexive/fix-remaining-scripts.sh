#!/bin/bash
#
# Fix remaining scripts with path issues
# This script updates path references in all L1 scripts
#

set -euo pipefail

# Get the absolute path to L1_reflexive
L1_DIR="$(cd "$(dirname "$0")" && pwd)"
HAL9_HOME="$(cd "$L1_DIR/.." && pwd)"

echo "Fixing paths in L1_reflexive scripts..."
echo "L1_DIR: $L1_DIR"
echo "HAL9_HOME: $HAL9_HOME"

# Function to fix common issues in a script
fix_script() {
    local script="$1"
    local temp_file="${script}.tmp"
    
    echo "Fixing: $script"
    
    # Skip if already fixed (has common-env.sh source)
    if grep -q "common-env.sh" "$script" 2>/dev/null; then
        echo "  Already fixed, skipping"
        return
    fi
    
    # Create backup
    cp "$script" "${script}.bak"
    
    # Start with shebang and header
    {
        echo '#!/bin/bash'
        echo '#'
        echo "# $(basename "$script" .sh | tr '-' ' ' | sed 's/\b\(.\)/\u\1/g')"
        echo '# Auto-fixed by L1 migration script'
        echo '#'
        echo ''
        echo 'set -euo pipefail'
        echo ''
        echo '# Source common environment'
        echo 'source "$(dirname "$0")/../../common-env.sh"'
        echo ''
        echo '# Original script content (modified for new paths)'
        echo ''
    } > "$temp_file"
    
    # Process the original script, skipping the shebang
    tail -n +2 "$script" | \
    sed 's|examples/|$HAL9_CONFIG_DIR/|g' | \
    sed 's|cargo run --bin hal9-server|$HAL9_SERVER_CMD|g' | \
    sed 's|cargo run --bin hal9-cli|$HAL9_CLI_CMD|g' | \
    sed 's|cargo run --bin hal9-codegen|$HAL9_CODEGEN_CMD|g' | \
    sed 's|cargo run --bin hal9|$HAL9_CLI_CMD|g' | \
    sed 's|./target/release/hal9-server|$HAL9_SERVER_BIN|g' | \
    sed 's|./target/debug/hal9-server|$HAL9_SERVER_BIN|g' | \
    sed 's|./target/release/hal9|$HAL9_CLI_BIN|g' | \
    sed 's|./target/debug/hal9|$HAL9_CLI_BIN|g' | \
    sed 's|data/hal9|$HAL9_DATA_DIR/hal9|g' | \
    sed 's|localhost:8080|localhost:$HAL9_PORT_MAIN|g' | \
    sed 's|echo "Error:|log_error "|g' | \
    sed 's|echo "Warning:|log_warning "|g' | \
    sed 's|echo "Info:|log_info "|g' | \
    sed 's|echo "✓|log_success "|g' | \
    sed 's|echo "✗|log_error "|g' | \
    sed 's|echo "⚠|log_warning "|g' >> "$temp_file"
    
    # Move temp file to original
    mv "$temp_file" "$script"
    chmod +x "$script"
    
    echo "  Fixed!"
}

# Fix emergency scripts
echo
echo "Fixing emergency scripts..."
for script in "$L1_DIR"/emergency/scripts/*.sh; do
    if [ -f "$script" ] && [ "$(basename "$script")" != "test-3neuron-demo.sh" ] && [ "$(basename "$script")" != "test-auth.sh" ]; then
        fix_script "$script"
    fi
done

# Fix response scripts
echo
echo "Fixing response scripts..."
for script in "$L1_DIR"/responses/scripts/*.sh; do
    if [ -f "$script" ]; then
        fix_script "$script"
    fi
done

# Fix status scripts
echo
echo "Fixing status scripts..."
for script in "$L1_DIR"/status/scripts/*.sh; do
    if [ -f "$script" ] && [ "$(basename "$script")" != "health-check.sh" ]; then
        fix_script "$script"
    fi
done

echo
echo "Creating quick test script..."
cat > "$L1_DIR/test-l1-fixes.sh" <<'EOF'
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
EOF

chmod +x "$L1_DIR/test-l1-fixes.sh"

echo
echo "Done! To test the fixes, run:"
echo "  $L1_DIR/test-l1-fixes.sh"
echo
echo "Backup files created with .bak extension"