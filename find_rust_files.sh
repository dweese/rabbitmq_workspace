#!/bin/bash

# Script to find all Rust (.rs) files in the workspace with full paths
# Usage: ./find_rust_files.sh

echo "=== Rust Files in Workspace ==="
echo "Current directory: $(pwd)"
echo "Timestamp: $(date)"
echo ""

# Find all .rs files and display with full paths
find . -name "*.rs" -type f | while read -r file; do
    # Get full path
    full_path=$(realpath "$file")

    # Get file size and modification time
    stat_info=$(stat -c "%s %Y" "$file")
    size=$(echo $stat_info | cut -d' ' -f1)
    mtime=$(echo $stat_info | cut -d' ' -f2)
    mod_date=$(date -d "@$mtime" "+%b %d %H:%M")

    # Display formatted output
    printf "%-60s %8s bytes %s\n" "$full_path" "$size" "$mod_date"
done | sort

echo ""
echo "=== Summary ==="
total_files=$(find . -name "*.rs" -type f | wc -l)
total_size=$(find . -name "*.rs" -type f -exec stat -c "%s" {} \; | awk '{sum+=$1} END {print sum}')
echo "Total Rust files: $total_files"
echo "Total size: $total_size bytes"

# Alternative more detailed view
echo ""
echo "=== Detailed Tree View ==="
find . -name "*.rs" -type f | sed 's|^\./||' | sort | while read -r file; do
    depth=$(echo "$file" | tr -cd '/' | wc -c)
    indent=$(printf "%*s" $((depth * 2)) "")
    basename=$(basename "$file")
    dirname=$(dirname "$file")

    if [ "$dirname" != "." ]; then
        echo "${indent}${dirname}/${basename}"
    else
        echo "${indent}${basename}"
    fi
done