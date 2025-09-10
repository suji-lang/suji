#!/bin/bash

# Script to verify spec tests
# Usage: ./scripts/verify_spec.sh

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

echo "Running spec tests..."
echo "===================="

# Find all .nn files in spec/ directory
for spec_file in spec/*.nn; do
    if [ ! -f "$spec_file" ]; then
        echo "No spec files found in spec/ directory"
        exit 1
    fi

    TOTAL=$((TOTAL + 1))
    filename=$(basename "$spec_file")

    # Extract expected output from the last line using perl with non-greedy pattern
    # Look for comment starting with "# " at the end of the last line
    expected_output=$(tail -n 1 "$spec_file" | perl -pe 's/.*?# (.*)/$1/')

    # Run the spec file and capture output
    actual_output=$(./target/release/nnlang "$spec_file" 2>/dev/null | tail -n 1)

    # Compare actual vs expected
    if [ "$actual_output" = "$expected_output" ]; then
        echo -e "${GREEN}PASS: $filename${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}FAIL: $filename - Expected '$expected_output', got '$actual_output'${NC}"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "===================="
echo "Spec Test Results:"
echo "  Total: $TOTAL"
echo -e "  ${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "  ${RED}Failed: $FAILED${NC}"
    exit 1
else
    echo -e "  ${GREEN}Failed: $FAILED${NC}"
fi
