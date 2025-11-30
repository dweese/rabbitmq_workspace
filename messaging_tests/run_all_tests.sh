#!/bin/bash

# This script finds and runs all test scripts in the 'scripts' directory.

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TEST_SCRIPT_DIR="$SCRIPT_DIR/scripts"

# Find all test scripts
TEST_FILES=$(find "$TEST_SCRIPT_DIR" -name "test_*.sh")

if [ -z "$TEST_FILES" ]; then
    echo "No test scripts found in $TEST_SCRIPT_DIR"
    exit 0
fi

FAILED_TESTS=0
PASSED_TESTS=0
TOTAL_TESTS=0

# Prompt for password once and export it for all sub-scripts
# This avoids being prompted for every single test script.
read -s -p "Enter RabbitMQ Password for all tests: " RABBITMQ_PASSWORD
export RABBITMQ_PASSWORD
echo

for test_file in $TEST_FILES; do
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo "================================================="
    echo "Running test: $test_file"
    echo "================================================="

    # Make sure the test script is executable, just in case
    chmod +x "$test_file"

    # Run the test
    if "$test_file"; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "-------------------------------------------------"
        echo "Test FAILED: $test_file"
        echo "-------------------------------------------------"
    fi
    echo
done

# Unset the password variable for security
unset RABBITMQ_PASSWORD

echo "================================================="
echo "Test Summary"
echo "================================================="
echo "Total tests run: $TOTAL_TESTS"
echo "Passed: $PASSED_TESTS"
echo "Failed: $FAILED_TESTS"
echo "================================================="

# Exit with a non-zero status code if any tests failed
if [ $FAILED_TESTS -ne 0 ]; then
    exit 1
fi

exit 0
