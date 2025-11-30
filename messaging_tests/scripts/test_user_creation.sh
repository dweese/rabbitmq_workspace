#!/bin/bash

# A script to test the user.create message flow.

# --- Configuration ---
QUEUE_NAME="user.create"
EXCHANGE_NAME="commands"
ROUTING_KEY="user.create"
PAYLOAD='{"email":"test@example.com","name":"Test User"}'
CONSUMER_OUTPUT_FILE="/tmp/consumer_output.txt"

# --- Helper Functions ---
function cleanup {
    echo "--- Cleaning up ---"
    rm -f "$CONSUMER_OUTPUT_FILE"
    unset RABBITMQ_PASSWORD # Unset the env var
}

# --- Main Test Logic ---
trap cleanup EXIT

echo "--- Test: User Creation Message ---"

# Prompt for password once and export it
if [ -z "$RABBITMQ_PASSWORD" ]; then
    read -s -p "Enter RabbitMQ Password: " RABBITMQ_PASSWORD
    export RABBITMQ_PASSWORD
    echo
fi

echo "Starting consumer in the background..."
# The consumer will now pick up the password from the environment
cargo run -p messaging_tests --bin test-consumer -- --queue "$QUEUE_NAME" > "$CONSUMER_OUTPUT_FILE" &
CONSUMER_PID=$!

# Give the consumer a moment to start up and connect
sleep 2

echo "Producing test message..."
# The producer will also pick up the password from the environment
cargo run -p messaging_tests --bin test-producer -- \
    --exchange "$EXCHANGE_NAME" \
    --routing-key "$ROUTING_KEY" \
    --payload "$PAYLOAD"

echo "Waiting for consumer to finish..."
wait $CONSUMER_PID

echo "Verifying output..."

if grep -q "$PAYLOAD" "$CONSUMER_OUTPUT_FILE"; then
    echo "✅ Test PASSED"
    exit 0
else
    echo "❌ Test FAILED"
    echo "Expected to find: $PAYLOAD"
    echo "But got:"
    cat "$CONSUMER_OUTPUT_FILE"
    exit 1
fi
