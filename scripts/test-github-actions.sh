#!/bin/bash
# Test GitHub Actions workflow files locally using act

set -e

echo "Testing GitHub Actions workflows locally..."

# Check if act is installed
if ! command -v act &> /dev/null; then
    echo "Error: 'act' is not installed."
    echo "Install it with: brew install act (macOS) or see https://github.com/nektos/act"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo "Error: Docker is not running. Please start Docker."
    exit 1
fi

# Test CI workflow
echo ""
echo "=== Testing CI Workflow ==="
act -W .github/workflows/ci.yml -j lint --dryrun

# Test security workflow  
echo ""
echo "=== Testing Security Workflow ==="
act -W .github/workflows/security.yml -j audit --dryrun

# Validate all workflow files
echo ""
echo "=== Validating Workflow Syntax ==="
for workflow in .github/workflows/*.yml; do
    echo -n "Checking $workflow... "
    if python3 -m yaml < "$workflow" > /dev/null 2>&1; then
        echo "✓ Valid"
    else
        echo "✗ Invalid YAML"
        exit 1
    fi
done

echo ""
echo "✓ All workflow files are valid!"
echo ""
echo "To run a workflow locally:"
echo "  act -W .github/workflows/ci.yml"
echo ""
echo "To see what would run:"
echo "  act -W .github/workflows/ci.yml --dryrun"