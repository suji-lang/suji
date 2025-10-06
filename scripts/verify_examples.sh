#!/bin/bash

# Script to verify example programs
# Usage: ./scripts/verify_examples.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
TOTAL=0
PASSED=0
FAILED=0

echo "Running example programs..."
echo "=========================="

# Find all .si files in examples/ directory
for example_file in examples/*.si; do
    if [ ! -f "$example_file" ]; then
        echo "No example files found in examples/ directory"
        exit 1
    fi

    TOTAL=$((TOTAL + 1))
    filename=$(basename "$example_file")

    # Run the example file and capture both stdout and stderr
    if ./target/release/suji "$example_file" >/dev/null 2>&1; then
        echo -e "${GREEN}PASS: $filename${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}FAIL: $filename - Program failed to execute${NC}"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "=========================="
echo "Example Test Results:"
echo "  Total: $TOTAL"
echo -e "  ${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "  ${RED}Failed: $FAILED${NC}"
    exit 1
else
    echo -e "  ${GREEN}Failed: $FAILED${NC}"
fi
