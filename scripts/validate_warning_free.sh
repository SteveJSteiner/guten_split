#!/bin/bash

# Warning-Free Build Validation Script
# WHY: Ensures ALL development scenarios documented in manual-commands.md are warning-free
# This script enforces zero warnings across every possible dev scenario

set -e

echo "🔍 Starting comprehensive warning-free validation..."
echo "   Testing ALL scenarios from docs/manual-commands.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track warnings across all scenarios
TOTAL_WARNINGS=0
FAILED_SCENARIOS=()

# Function to check for warnings in output
check_warnings() {
    local output="$1"
    local scenario="$2"
    local warnings=0
    
    # Count warnings in output
    if echo "$output" | grep -q "warning:"; then
        warnings=$(echo "$output" | grep -c "warning:" || echo "0")
    fi
    
    TOTAL_WARNINGS=$((TOTAL_WARNINGS + warnings))
    
    if [ $warnings -gt 0 ]; then
        echo -e "${RED}❌ FAILED: $scenario ($warnings warnings)${NC}"
        FAILED_SCENARIOS+=("$scenario")
        echo "   Sample warnings:"
        echo "$output" | grep "warning:" | head -3 | sed 's/^/   /'
        echo ""
    else
        echo -e "${GREEN}✅ PASSED: $scenario${NC}"
    fi
}

# Function to run command and check warnings
run_and_check() {
    local cmd="$1"
    local scenario="$2"
    
    echo "📋 Testing: $scenario"
    
    # Capture both stdout and stderr
    local output
    if output=$(eval "$cmd" 2>&1); then
        check_warnings "$output" "$scenario"
    else
        echo -e "${RED}❌ FAILED: $scenario (command failed)${NC}"
        FAILED_SCENARIOS+=("$scenario")
        echo "   Error: $output"
    fi
}

echo ""
echo "🔧 Core Build Commands"
echo "==================="

# Standard Debug Build
run_and_check "cargo build" "Standard debug build"

# Release Build
run_and_check "cargo build --release" "Release build"

# Build with Features
run_and_check "cargo build --features mmap" "Build with mmap feature"

# Build Specific Binaries
run_and_check "cargo build --bin seams" "Build seams binary"
run_and_check "cargo build --bin generate_boundary_tests" "Build generate_boundary_tests binary"
run_and_check "cargo build --bin generate_gutenberg_sentences" "Build generate_gutenberg_sentences binary"

# Build Library Only
run_and_check "cargo build --lib" "Build library only"

echo ""
echo "🧪 Test Commands"
echo "==============="

# Run All Tests
run_and_check "cargo test" "Run all tests"

# Run Specific Test Types
run_and_check "cargo test --lib" "Unit tests only"
run_and_check "cargo test --test error_handling_integration" "Error handling integration tests"
run_and_check "cargo test --test incremental_processing_integration" "Incremental processing integration tests"
run_and_check "cargo test --test pipeline_integration" "Pipeline integration tests"
run_and_check "cargo test --doc" "Doc tests only"

echo ""
echo "🔍 Code Quality"
echo "==============="

# Check Code (No Build)
run_and_check "cargo check" "Code check (no build)"

# Lint with Clippy
run_and_check "cargo clippy" "Clippy linting"

# Clippy with deny warnings
run_and_check "cargo clippy -- -D warnings" "Clippy with deny warnings"

echo ""
echo "🎯 Feature Matrix Testing"
echo "========================="

# Test All Feature Combinations
run_and_check "cargo test --features mmap" "Test with mmap feature"
run_and_check "cargo test --features test-helpers" "Test with test-helpers feature"
run_and_check "cargo test --features \"mmap,test-helpers\"" "Test with multiple features"
run_and_check "cargo test --all-features" "Test with all features"
run_and_check "cargo test --no-default-features" "Test with no default features"

# Build All Feature Combinations
run_and_check "cargo build --all-features" "Build with all features"
run_and_check "cargo build --no-default-features" "Build with no default features"

# Clippy All Feature Combinations
run_and_check "cargo clippy --all-features" "Clippy with all features"
run_and_check "cargo clippy --no-default-features" "Clippy with no default features"

echo ""
echo "📊 Final Results"
echo "================"

if [ $TOTAL_WARNINGS -eq 0 ] && [ ${#FAILED_SCENARIOS[@]} -eq 0 ]; then
    echo -e "${GREEN}🎉 SUCCESS: All scenarios are warning-free!${NC}"
    echo "   ✅ Zero warnings across ALL development scenarios"
    echo "   ✅ All commands executed successfully"
    exit 0
else
    echo -e "${RED}❌ FAILURE: $TOTAL_WARNINGS warnings found across ${#FAILED_SCENARIOS[@]} scenarios${NC}"
    echo ""
    echo "Failed scenarios:"
    for scenario in "${FAILED_SCENARIOS[@]}"; do
        echo "  - $scenario"
    done
    echo ""
    echo "🔧 To fix warnings, run:"
    echo "   cargo clippy --fix --allow-dirty --allow-staged"
    echo "   cargo clippy --all-features --fix --allow-dirty --allow-staged"
    echo ""
    exit 1
fi