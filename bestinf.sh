#!/bin/bash
# Enhanced project overview script

echo "=== PROJECT STRUCTURE ==="
tree -I 'target|node_modules|.git' -a -L 3  # Limit depth if large

echo -e "\n=== RUST FILES (recent modifications) ==="
find . -name "*.rs" -o -name "Cargo.toml" | xargs ls -lth | head -20

echo -e "\n=== WORKSPACE CARGO.TOML ==="
cat Cargo.toml

echo -e "\n=== CRATE CARGO.TOMLS ==="
cat */Cargo.toml 2>/dev/null

echo -e "\n=== LIBRARY ENTRY POINTS ==="
for lib in */src/lib.rs; do
  echo "--- $lib ---"
  head -30 "$lib"  # Maybe 30 lines for better context?
done

echo -e "\n=== MAIN BINARIES (if any) ==="
find . -path "*/src/main.rs" -exec echo "--- {} ---" \; -exec head -20 {} \;
