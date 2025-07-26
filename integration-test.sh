#!/bin/bash

# Integration Test Script for EDSL to Excalidraw Conversion
set -e

echo "🧪 EDSL Integration Test Suite"
echo "==============================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test directories
TEST_DIR="test-output"
mkdir -p "$TEST_DIR"

# Function to test API endpoints
test_api_endpoint() {
    local edsl_content="$1"
    local test_name="$2"
    
    echo -e "${YELLOW}🔍 Testing: $test_name${NC}"
    
    # Test HTTP API
    local response=$(curl -s -X POST http://localhost:3002/api/compile \
        -H "Content-Type: application/json" \
        -d "{\"edsl_content\": $(echo "$edsl_content" | jq -R -s .)}")
    
    # Check if compilation was successful
    local success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "true" ]; then
        echo -e "${GREEN}✅ Compilation successful${NC}"
        
        # Save and validate output
        echo "$response" | jq '.data' > "$TEST_DIR/${test_name}.json"
        
        # Check element count and structure
        local element_count=$(echo "$response" | jq '.data | length')
        echo -e "${BLUE}📊 Generated $element_count elements${NC}"
        
        # Validate required Excalidraw properties
        local valid=true
        for i in $(seq 0 $((element_count - 1))); do
            local element=$(echo "$response" | jq ".data[$i]")
            if ! echo "$element" | jq -e 'has("id") and has("type") and has("x") and has("y")' >/dev/null; then
                echo -e "${RED}❌ Element $i missing required properties${NC}"
                valid=false
            fi
        done
        
        if [ "$valid" = true ]; then
            echo -e "${GREEN}✅ All elements have required Excalidraw properties${NC}"
            return 0
        else
            return 1
        fi
    else
        local error=$(echo "$response" | jq -r '.error')
        echo -e "${RED}❌ Compilation failed: $error${NC}"
        return 1
    fi
}

# Check server health
echo -e "${BLUE}🏥 Checking server health${NC}"
if ! curl -s http://localhost:3002/health >/dev/null; then
    echo -e "${RED}❌ Server not responding. Please start the server first.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Server is running${NC}"

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

echo -e "\n${BLUE}🧪 Running integration tests${NC}"

# Test 1: Simple nodes
SIMPLE_EDSL="a[Node A]
b[Node B]
a -> b"

if test_api_endpoint "$SIMPLE_EDSL" "simple"; then
    ((TESTS_PASSED++))
else
    ((TESTS_FAILED++))
fi
echo ""

# Test 2: Nodes with styles
STYLED_EDSL="---
layout: dagre
---

start[Start] {
  strokeColor: \"#22c55e\";
  backgroundColor: \"#dcfce7\";
}

end[End] {
  strokeColor: \"#ef4444\";
  backgroundColor: \"#fee2e2\";
}

start -> end"

if test_api_endpoint "$STYLED_EDSL" "styled"; then
    ((TESTS_PASSED++))
else
    ((TESTS_FAILED++))
fi
echo ""

# Test 3: Edge labels with fixed syntax
LABELED_EDSL="question[Question]
yes[Yes]
no[No]

question -> yes{Yes}
question -> no{No}"

if test_api_endpoint "$LABELED_EDSL" "labeled"; then
    ((TESTS_PASSED++))
else
    ((TESTS_FAILED++))
fi
echo ""

# Test 4: Complex flow (acyclic graph)
COMPLEX_EDSL="---
layout: dagre
---

start[Start]
process[Process Data]
decision[Decision]
end[End]
error[Error]

start -> process
process -> decision{Decision?}
decision -> end{Success}
decision -> error{Error}"

if test_api_endpoint "$COMPLEX_EDSL" "complex"; then
    ((TESTS_PASSED++))
else
    ((TESTS_FAILED++))
fi
echo ""

# Summary
echo -e "\n${BLUE}📊 Test Summary${NC}"
echo "==============="
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"
echo -e "Total tests: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed!${NC}"
    echo -e "${GREEN}Generated Excalidraw files are valid and ready to use${NC}"
    exit 0
else
    echo -e "\n${RED}💥 Some tests failed!${NC}"
    exit 1
fi