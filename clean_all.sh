#!/bin/bash
# clean_all.sh - Clean all workspace members

set -e  # Exit on any error

echo "🧹 Cleaning RabbitMQ Workspace..."

# List of workspace members from your Cargo.toml
PROJECTS=(
    "rabbitmq-config"
    "rabbitmq-info" 
    "rabbitmq-ui"
    "rabbitmq-mon"
    "egui-components"
    "messaging_commands"
    "messaging_cli"
    "yak_json"
    "pg_vault"
)

# Clean root workspace first
echo "🗑️  Cleaning root workspace..."
cargo clean

# Clean each subproject
for project in "${PROJECTS[@]}"; do
    if [ -d "$project" ]; then
        echo "🗑️  Cleaning $project..."
        cd "$project"
        cargo clean
        cd ..
    else
        echo "⚠️  Warning: $project directory not found"
    fi
done

# Clean any additional Cargo artifacts
echo "🗑️  Cleaning additional artifacts..."
rm -rf target/
find . -name "Cargo.lock" -not -path "./Cargo.lock" -delete 2>/dev/null || true
find . -name "target" -type d -exec rm -rf {} + 2>/dev/null || true

echo "✅ All projects cleaned!"
echo "📊 Disk space freed:"
du -sh . 2>/dev/null || echo "Unable to calculate size"