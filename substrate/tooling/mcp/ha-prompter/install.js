#!/usr/bin/env node

/**
 * Post-install script for ha-prompter
 * Downloads the appropriate binary for the current platform
 */

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const REPO = 'hal9000/ha-prompter';
const VERSION = require('./package.json').version;

// Determine platform and architecture
const platform = process.platform;
const arch = process.arch;

// Map Node platform/arch to Rust target triples
const targetMap = {
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'linux-arm64': 'aarch64-unknown-linux-gnu',
  'win32-x64': 'x86_64-pc-windows-msvc',
};

const target = targetMap[`${platform}-${arch}`];

if (!target) {
  console.error(`Unsupported platform: ${platform}-${arch}`);
  console.error('Falling back to cargo install...');
  
  try {
    execSync('cargo --version', { stdio: 'ignore' });
    console.log('Building from source with cargo...');
    execSync('cargo install ha-prompter', { stdio: 'inherit' });
    process.exit(0);
  } catch (e) {
    console.error('Cargo not found. Please install Rust: https://rustup.rs');
    process.exit(1);
  }
}

const binaryName = platform === 'win32' ? 'ha-prompter.exe' : 'ha-prompter';
const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/ha-prompter-${VERSION}-${target}.tar.gz`;

const binDir = path.join(__dirname, 'bin');
const binPath = path.join(binDir, binaryName);

// Create bin directory
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

console.log(`Downloading ha-prompter ${VERSION} for ${platform}-${arch}...`);
console.log(`URL: ${downloadUrl}`);

// Download and extract binary
const download = (url, dest) => {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    
    https.get(url, (response) => {
      if (response.statusCode === 302) {
        // Handle redirect
        return download(response.headers.location, dest).then(resolve).catch(reject);
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }
      
      response.pipe(file);
      
      file.on('finish', () => {
        file.close(resolve);
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
  });
};

const extractTarGz = (src, dest) => {
  // Extract using tar command
  execSync(`tar -xzf ${src} -C ${dest}`, { stdio: 'inherit' });
};

async function install() {
  try {
    const tempFile = path.join(binDir, 'temp.tar.gz');
    
    // Download
    await download(downloadUrl, tempFile);
    
    // Extract
    extractTarGz(tempFile, binDir);
    
    // Clean up
    fs.unlinkSync(tempFile);
    
    // Make executable
    if (platform !== 'win32') {
      fs.chmodSync(binPath, 0o755);
    }
    
    console.log('✅ ha-prompter installed successfully!');
    console.log(`📍 Binary location: ${binPath}`);
    
  } catch (error) {
    console.error('❌ Installation failed:', error.message);
    console.error('\nFalling back to cargo install...');
    
    try {
      execSync('cargo install ha-prompter', { stdio: 'inherit' });
    } catch (e) {
      console.error('Please install manually with: cargo install ha-prompter');
      process.exit(1);
    }
  }
}

// Run installation
install();