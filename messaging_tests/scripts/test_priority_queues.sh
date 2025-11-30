#!/bin/bash

# A script to test that priority queues process higher priority messages first.

# --- Configuration ---
QUEUE_NAME="order.placed" # This is a priority queue (max-priority=5 in our spec)
EXCHANGE_NAME="events"
ROUTING_KEY="order.placed"

LOW_PRIORITY_PAYLOAD='{"order_id":"low-priority-order"}'
HIGH_PRIORITY_PAYLOAD='{"order_id":"high-priority-order"}'

FIRST_CONSUMER_OUTPUT_FILE="/tmp/first_consumer_output.txt"
SECOND_CONSUMER_OUTPUT_FILE="/tmp/second_consumer_output.txt"

# --- Helper Functions ---
function cleanup {
    echo "--- Cleaning up ---"
    rm -f "$FIRST_CONSUMER_OUTPUT_FILE"
    rm -f "$SECOND_CONSUMER_OUTPUT_FILE"
    unset RABBITMQ_PASSWORD # Unset the env var
}

# --- Main Test Logic ---
trap cleanup EXIT

echo "--- Test: Priority Queue Message Ordering ---"

# Prompt for password once and export it
if [ -z "$RABBITMQ_PASSWORD" ]; then
    read -s -p "Enter RabbitMQ Password: " RABBITMQ_PASSWORD
    export RABBITMQ_PASSWORD
    echo
fi

echo "Producing two messages out of order (low priority first)..."

# Send a LOW priority message
cargo run -p messaging_tests --bin test-producer -- \
    --exchange "$EXCHANGE_NAME" \
    --routing-key "$ROUTING_KEY" \
    --payload "$LOW_PRIORITY_PAYLOAD" \
    --priority 1

# Send a HIGH priority message
cargo run -p messaging_tests --bin test-producer -- \
    --exchange "$EXCHANGE_NAME" \
    --routing-key "$ROUTING_KEY" \
    --payload "$HIGH_PRIORITY_PAYLOAD" \
    --priority 5 # Max priority for this queue

echo "Consuming the first message..."
cargo run -p messaging_tests --bin test-consumer -- --queue "$QUEUE_NAME" > "$FIRST_CONSUMER_OUTPUT_FILE"

echo "Consuming the second message..."
cargo run -p messaging_tests --bin test-consumer -- --queue "$QUEUE_NAME" > "$SECOND_CONSUMER_OUTPUT_FILE"

echo "Verifying output..."

# The first message consumed should be the HIGH priority one
if ! grep -q "$HIGH_PRIORITY_PAYLOAD" "$FIRST_CONSUMER_OUTPUT_FILE"; then
    echo "❌ Test FAILED: High priority message was not received first."
    echo "Expected first message: $HIGH_PRIORITY_PAYLOAD"
    echo "Received first message:"
    cat "$FIRST_CONSUMER_OUTPUT_FILE"
    exit 1
fi

# The second message consumed should be the LOW priority one
if ! grep -q "$LOW_PRIORITY_PAYLOAD" "$SECOND_CONSUMER_OUTPUT_FILE"; then
    echo "❌ Test FAILED: Low priority message was not received second."
    echo "Expected second message: $LOW_PRIORITY_PAYLOAD"
    echo "Received second message:"
    cat "$SECOND_CONSUMER_OUTPUT_FILE"
    exit 1
fi

echo "✅ Test PASSED: Messages received in correct priority order."
exit 0
