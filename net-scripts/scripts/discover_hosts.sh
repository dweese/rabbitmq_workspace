#!/bin/bash

# A script to discover live hosts on a network.
# It uses the first command-line argument ($1) if provided.
# If not, it falls back to the DEFAULT_SUBNET environment variable.

TARGET_SUBNET="${1:-$DEFAULT_SUBNET}"

# Check if a target subnet is available from either source.
if [ -z "$TARGET_SUBNET" ]; then
  echo "Error: No subnet provided and DEFAULT_SUBNET environment variable is not set."
  echo "Usage: $0 [subnet]"
  exit 1
fi

echo "Scanning target: ${TARGET_SUBNET}"
nmap -sn "${TARGET_SUBNET}"