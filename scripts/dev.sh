#!/bin/bash

# Chesstack Development Server
# Quick script to start the development server

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if we're in the project root
if [ -d "rust" ]; then
    cd rust
elif [ ! -f "index.html" ]; then
    echo -e "${YELLOW}Warning: index.html not found. Are you in the right directory?${NC}"
fi

echo -e "${GREEN}Starting Chesstack development server...${NC}"
echo -e "${YELLOW}Open http://localhost:8080/index.html in your browser${NC}"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

python3 -m http.server 8080
