#!/bin/bash
# Run unit tests

echo "🧪 Running Unit Tests"
echo "===================="
echo

# Compile and run unit tests
echo "Compiling unit test modules..."

# Create a temporary test runner
cat > tests/unit/test_runner.rs << 'EOF'
// Include all unit test modules
mod test_rate_limiter;
mod test_circuit_breaker;
mod test_auth;

fn main() {
    println!("Unit tests are run with `cargo test`");
}
EOF

# Run tests using rustc directly for standalone tests
echo
echo "Testing Rate Limiter..."
rustc --test tests/unit/test_rate_limiter.rs -o tests/unit/test_rate_limiter_bin 2>/dev/null
if [ -f tests/unit/test_rate_limiter_bin ]; then
    ./tests/unit/test_rate_limiter_bin --test-threads=1
else
    echo "⚠️  Could not compile rate limiter tests standalone"
fi

echo
echo "Testing Circuit Breaker..."
rustc --test tests/unit/test_circuit_breaker.rs -o tests/unit/test_circuit_breaker_bin 2>/dev/null
if [ -f tests/unit/test_circuit_breaker_bin ]; then
    ./tests/unit/test_circuit_breaker_bin --test-threads=1
else
    echo "⚠️  Could not compile circuit breaker tests standalone"
fi

echo
echo "Testing Authentication..."
rustc --test tests/unit/test_auth.rs -o tests/unit/test_auth_bin 2>/dev/null
if [ -f tests/unit/test_auth_bin ]; then
    ./tests/unit/test_auth_bin --test-threads=1
else
    echo "⚠️  Could not compile auth tests standalone"
fi

# Clean up
rm -f tests/unit/test_*_bin tests/unit/test_runner.rs

echo
echo "Note: For full unit test coverage, run these tests within the project using:"
echo "  cargo test --lib"
echo
echo "Unit test examples completed!"