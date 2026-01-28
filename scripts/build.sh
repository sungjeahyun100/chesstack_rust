#!/bin/bash

# Chesstack Build Script
# This script builds the entire Chesstack project including Rust code and WASM

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Chesstack Build Script ===${NC}"
echo ""

# Check if we're in the project root
if [ ! -d "rust" ]; then
    echo -e "${RED}Error: Please run this script from the project root directory${NC}"
    exit 1
fi

# Check for Rust installation
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo not found. Please install Rust first.${NC}"
    echo "Visit: https://rustup.rs/"
    exit 1
fi

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${YELLOW}Warning: wasm-pack not found. Installing...${NC}"
    cargo install wasm-pack
fi

# Check for wasm32 target
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo -e "${YELLOW}Warning: wasm32-unknown-unknown target not installed. Adding...${NC}"
    rustup target add wasm32-unknown-unknown
fi

# Step 1: Build and test Rust code
echo -e "${GREEN}Step 1/3: Building and testing Rust code...${NC}"
cd rust
cargo build --release
echo ""

echo -e "${GREEN}Running tests...${NC}"
cargo test
echo ""

# Step 2: Build WASM package
echo -e "${GREEN}Step 2/3: Building WebAssembly package...${NC}"
cd wasm
wasm-pack build --target web --out-dir ../pkg
cd ..
echo ""

# Step 3: Summary
echo -e "${GREEN}Step 3/3: Build complete!${NC}"
echo ""
echo -e "${GREEN}=== Build Summary ===${NC}"
echo "✓ Rust code built successfully"
echo "✓ Tests passed"
echo "✓ WASM package generated at rust/pkg/"
echo ""
echo -e "${YELLOW}To start the web server:${NC}"
echo "  cd rust"
echo "  python3 -m http.server 8080"
echo ""
echo -e "${YELLOW}Then open:${NC}"
echo "  http://localhost:8080/index.html"
echo ""
