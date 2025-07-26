#!/bin/bash

echo "🚀 Starting ExcaliDraw DSL backend server..."
echo "========================================"

# Build the server if needed
echo "Building server..."
cargo build --bin edsl-server --features server

if [ $? -eq 0 ]; then
    echo "✅ Build successful"
    echo ""
    echo "Starting server on port 3002..."
    echo "Health check will be available at: http://localhost:3002/health"
    echo "WebSocket endpoint: ws://localhost:3002/api/ws"
    echo ""
    echo "Press Ctrl+C to stop the server"
    echo ""
    
    # Start the server
    cargo run --bin edsl-server --features server -- --port 3002 --verbose
else
    echo "❌ Build failed"
    exit 1
fi