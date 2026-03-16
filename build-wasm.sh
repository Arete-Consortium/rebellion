#!/bin/bash
# Build Rebellion for WebAssembly

set -e

echo "Building Rebellion for WASM..."

# Check for wasm-bindgen-cli
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Build for WASM
echo "Compiling for wasm32-unknown-unknown..."
cargo build --release --target wasm32-unknown-unknown

# Generate JS bindings
echo "Generating JavaScript bindings..."
wasm-bindgen \
    --out-dir web \
    --target web \
    target/wasm32-unknown-unknown/release/rebellion.wasm

# Copy assets
echo "Copying assets..."
cp -r assets web/

# Optimize WASM (optional, requires wasm-opt from binaryen)
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM..."
    wasm-opt -Oz -o web/rebellion_bg.wasm web/rebellion_bg.wasm
else
    echo "Note: Install binaryen for WASM optimization (wasm-opt)"
fi

echo ""
echo "Build complete! Files are in the 'web' directory."
echo ""
echo "To run locally:"
echo "  cd web && python3 -m http.server 8080"
echo "  Then open http://localhost:8080"
echo ""
