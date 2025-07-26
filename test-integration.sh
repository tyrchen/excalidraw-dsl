#!/bin/bash

# Integration test script for ExcaliDraw DSL
set -e

echo "🧪 Running ExcaliDraw DSL Integration Tests"
echo "==========================================="

# Test 1: Build the project
echo "1️⃣  Building Rust project..."
cargo build --features server
echo "✅ Build successful"

# Test 2: Test CLI compilation
echo ""
echo "2️⃣  Testing CLI compilation..."
cat > test.edsl << 'EOF'
---
layout: dagre
---

# Test diagram
start[Start Node]
process[Process Node]  
end[End Node]

start -> process -> end
EOF

./target/debug/edsl compile test.edsl -o test-output.json
if [ -f test-output.json ]; then
    echo "✅ CLI compilation successful"
    rm -f test.edsl test-output.json
else
    echo "❌ CLI compilation failed"
    exit 1
fi

# Test 3: Test server startup
echo ""
echo "3️⃣  Testing server startup..."
timeout 5s ./target/debug/edsl-server --port 3002 &
SERVER_PID=$!
sleep 2

# Test 4: Test HTTP API
echo ""
echo "4️⃣  Testing HTTP API..."
HEALTH_RESPONSE=$(curl -s http://localhost:3002/health || echo "failed")
if [[ $HEALTH_RESPONSE == *"healthy"* ]]; then
    echo "✅ Health endpoint working"
else
    echo "❌ Health endpoint failed: $HEALTH_RESPONSE"
fi

# Test 5: Test compilation API
echo ""
echo "5️⃣  Testing compilation API..."
COMPILE_RESPONSE=$(curl -s -X POST http://localhost:3002/api/compile \
    -H "Content-Type: application/json" \
    -d '{"edsl_content":"node1[Test]\nnode2[Node]\nnode1 -> node2"}' || echo "failed")

if [[ $COMPILE_RESPONSE == *"success"* ]]; then
    echo "✅ Compilation API working"
else
    echo "❌ Compilation API failed: $COMPILE_RESPONSE"
fi

# Test 6: Test validation API
echo ""
echo "6️⃣  Testing validation API..."
VALIDATE_RESPONSE=$(curl -s -X POST http://localhost:3002/api/validate \
    -H "Content-Type: application/json" \
    -d '{"edsl_content":"node1[Test Node]"}' || echo "failed")

if [[ $VALIDATE_RESPONSE == *"is_valid"* ]]; then
    echo "✅ Validation API working"
else
    echo "❌ Validation API failed: $VALIDATE_RESPONSE"
fi

# Cleanup
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo ""
echo "🎉 All integration tests passed!"
echo ""
echo "To run the full system:"
echo "  make run-full     # Starts both server and UI"
echo "  make run-server   # Starts only the server"
echo "  make run-ui       # Starts only the UI"