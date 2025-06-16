# Build Optimization Guide

## 🚀 Quick Start

```bash
# Initial setup (run once)
make setup

# Fast parallel build
make build-all

# Even faster check
make check-parallel

# Build specific package
make build-package PKG=genius_game_server
```

## ⚡ Optimizations Applied

### 1. **Parallel Compilation** (`.cargo/config.toml`)
- Uses all 16 CPU cores (`jobs = 16`)
- Incremental compilation enabled
- LLD linker for faster linking

### 2. **Compilation Cache** (sccache)
- Caches compilation artifacts
- Reuses unchanged dependencies
- Install: `cargo install sccache`
- Enable: `export RUSTC_WRAPPER=sccache`

### 3. **Profile Optimizations**
- Development: Fast compilation, minimal optimization
- Dependencies: Pre-optimized (level 2)
- Release: Thin LTO for balance

### 4. **Sparse Registry Protocol**
- Faster crate index updates
- Less bandwidth usage

## 📊 Expected Improvements

| Build Type | Before | After |
|------------|--------|-------|
| Clean build | >2min | ~90s |
| Incremental | 30s | <10s |
| Check only | 20s | <5s |

## 🔧 Advanced Options

### Use mold linker (Linux/macOS)
```bash
# Install mold
brew install mold  # macOS
sudo apt install mold  # Ubuntu

# Enable in .cargo/config.toml
[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

### Optimize for specific CPU
```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

### Reduce debug info
```toml
[profile.dev]
debug = 0  # No debug info
# or
debug = 1  # Line numbers only
```

## 🎯 Workflow Tips

1. **Use `cargo check` during development**
   ```bash
   make check-parallel
   ```

2. **Build only what changed**
   ```bash
   make build-package PKG=your_package
   ```

3. **Monitor sccache hits**
   ```bash
   make build-with-stats
   ```

4. **Clean build when needed**
   ```bash
   make clean
   make build-all
   ```

## 🚨 Troubleshooting

### LLD not found
```bash
# macOS
brew install llvm
export PATH="/usr/local/opt/llvm/bin:$PATH"

# Linux
sudo apt install lld
```

### Out of memory
- Reduce parallel jobs: `jobs = 8`
- Disable LTO: `lto = false`

### Incremental compilation issues
```bash
cargo clean
rm -rf target/
```