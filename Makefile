.PHONY: build-all build-release test clean bench check fmt clippy time-build

# Default target
all: build-all

# Build all targets and measure time
build-all:
	@echo "🚀 Building all targets with parallel optimization..."
	@echo "Build started at: $$(date)"
	@echo "CPU cores: $$(sysctl -n hw.ncpu)"
	@echo "Parallel jobs: $$CARGO_BUILD_JOBS"
	@start_time=$$(date +%s); \
	CARGO_BUILD_JOBS=16 cargo build --all-targets --all-features -j 16 && \
	end_time=$$(date +%s); \
	duration=$$((end_time - start_time)); \
	echo "✅ Build completed in $$duration seconds ($$((duration / 60))m $$((duration % 60))s)"

# Build release mode
build-release:
	@echo "🚀 Building release targets..."
	@start_time=$$(date +%s); \
	cargo build --release --all-targets --all-features && \
	end_time=$$(date +%s); \
	duration=$$((end_time - start_time)); \
	echo "✅ Release build completed in $$duration seconds"

# Time different build configurations
time-build:
	@echo "⏱️  Measuring build times..."
	@echo "\n1. Clean build:"
	@cargo clean
	@start_time=$$(date +%s); \
	cargo build --all-targets && \
	end_time=$$(date +%s); \
	echo "   Clean build: $$((end_time - start_time))s"
	@echo "\n2. Incremental build (no changes):"
	@start_time=$$(date +%s); \
	cargo build --all-targets && \
	end_time=$$(date +%s); \
	echo "   Incremental: $$((end_time - start_time))s"
	@echo "\n3. Check only:"
	@start_time=$$(date +%s); \
	cargo check --all-targets && \
	end_time=$$(date +%s); \
	echo "   Check only: $$((end_time - start_time))s"

# Run all tests
test:
	@echo "🧪 Running all tests..."
	cargo test --all

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Run benchmarks
bench:
	@echo "📊 Running benchmarks..."
	cargo bench

# Check code without building
check:
	@echo "🔍 Checking code..."
	cargo check --all

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all

# Run clippy linter
clippy:
	@echo "📎 Running clippy..."
	cargo clippy --all -- -W clippy::all

# Install build optimizations
install-optimizations:
	@echo "📦 Installing build optimizations..."
	@echo "Installing sccache..."
	cargo install sccache || true
	@echo "Installing cargo-nextest..."
	cargo install cargo-nextest || true
	@echo "Done! Set RUSTC_WRAPPER=sccache in your environment"

# Show system info
system-info:
	@echo "💻 System Information:"
	@echo "CPU cores: $$(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 'unknown')"
	@echo "Memory: $$(sysctl -n hw.memsize 2>/dev/null | awk '{print $$1/1024/1024/1024 " GB"}' || free -h 2>/dev/null | grep Mem | awk '{print $$2}' || echo 'unknown')"
	@rustc --version
	@cargo --version

# Fast incremental build (development)
build-fast:
	@echo "⚡ Fast incremental build..."
	@CARGO_BUILD_JOBS=16 CARGO_PROFILE_DEV_BUILD_OVERRIDE_OPT_LEVEL=0 cargo build -j 16

# Build specific package
build-package:
	@echo "📦 Building package: $(PKG)"
	@CARGO_BUILD_JOBS=16 cargo build -p $(PKG) -j 16

# Parallel check (faster than build)
check-parallel:
	@echo "🔍 Running parallel check..."
	@start_time=$$(date +%s); \
	CARGO_BUILD_JOBS=16 cargo check --all-targets -j 16 && \
	end_time=$$(date +%s); \
	duration=$$((end_time - start_time)); \
	echo "✅ Check completed in $$duration seconds"

# Setup build environment
setup:
	@./setup-build-optimizations.sh

# Build with sccache stats
build-with-stats:
	@echo "📊 Building with sccache statistics..."
	@sccache --show-stats || true
	@RUSTC_WRAPPER=sccache CARGO_BUILD_JOBS=16 cargo build --all-targets -j 16
	@echo "\n📈 Sccache statistics after build:"
	@sccache --show-stats || true