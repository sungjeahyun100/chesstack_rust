#!/bin/bash

# Chesstack Test Script
# Runs all tests for the project

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Running Chesstack Tests ===${NC}"
echo ""

cd rust

echo -e "${YELLOW}Running chessembly tests...${NC}"
cargo test --package chessembly
echo ""

echo -e "${YELLOW}Running engine tests...${NC}"
cargo test --package engine
echo ""

echo -e "${GREEN}All tests passed!${NC}"
