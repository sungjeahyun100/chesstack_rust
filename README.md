# Chesstack

A chess variant game with custom piece movement DSL (Chessembly).

## Requirements

- Ubuntu 24.04 LTS (or similar Linux distribution)
- Rust 1.93.0 or later
- wasm-pack 0.14.0 or later
- Python 3 (for local web server)

## Installation

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Install wasm-pack
```bash
cargo install wasm-pack
```

### Add WebAssembly target
```bash
rustup target add wasm32-unknown-unknown
```

## Build

### Quick build (all-in-one)
```bash
./scripts/build.sh
```

### Manual build steps

#### 1. Build and test Rust code
```bash
cd rust
cargo build --release
cargo test
```

#### 2. Build WebAssembly package
```bash
cd rust/wasm
wasm-pack build --target web --out-dir ../pkg
```

#### 3. Start web server
```bash
cd rust
python3 -m http.server 8080
```

Then open http://localhost:8080/index.html in your browser.

## Development

### Run tests
```bash
cd rust
cargo test
```

### Build in development mode
```bash
cd rust
cargo build
```

### Rebuild WASM only
```bash
cd rust/wasm
wasm-pack build --target web --out-dir ../pkg
```

## Project Structure

- `rust/chessembly/` - Chessembly DSL interpreter
- `rust/engine/` - Game logic and state management
- `rust/wasm/` - WebAssembly bindings
- `rust/pkg/` - Generated WASM package
- `rust/index.html` - Web UI
- `docs/` - Documentation
- `config/` - Configuration files

