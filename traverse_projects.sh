#!/bin/bash
# Rust Workspace Build Script
# Builds all Rust projects in the workspace

echo "=== Rust Workspace Builder ==="
echo "Working directory: $(pwd)"
echo "Date: $(date)"
echo

# Simple color codes
RED="\033[0;31m"
GREEN="\033[0;32m"
BLUE="\033[0;34m"
YELLOW="\033[1;33m"
NC="\033[0m"

# Simple logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Initialize counters
total=0
success=0
failed=0

# Save original directory
original_dir="$(pwd)"

log_info "Scanning for Rust projects..."

# List of project directories (excluding the workspace root)
projects=(
    "egui-components"
    "messaging_cli"
    "messaging_commands" 
    "pg_vault"
    "rabbitmq-config"
    "rabbitmq-info"
    "rabbitmq-ui"
    "yak_json"
)

# Process each project
for project in "${projects[@]}"; do
    if [ ! -d "$project" ]; then
        log_warning "Directory not found: $project"
        continue
    fi
    
    if [ ! -f "$project/Cargo.toml" ]; then
        log_warning "No Cargo.toml found in: $project"
        continue
    fi
    
    log_info "Processing project: $project"
    total=$((total + 1))
    
    # Change to project directory
    cd "$project" || {
        log_error "Failed to enter directory: $project"
        failed=$((failed + 1))
        continue
    }
    
    # Run cargo clean
    log_info "Running cargo clean in $project..."
    if cargo clean > /dev/null 2>&1; then
        log_success "cargo clean completed for $project"
        
        # Run cargo build
        log_info "Running cargo build in $project..."
        if cargo build > /dev/null 2>&1; then
            log_success "cargo build completed for $project"
            success=$((success + 1))
        else
            log_error "cargo build failed for $project"
            failed=$((failed + 1))
        fi
    else
        log_error "cargo clean failed for $project"
        failed=$((failed + 1))
    fi
    
    # Return to workspace
    cd "$original_dir" || {
        log_error "CRITICAL: Failed to return to workspace directory"
        exit 1
    }
    
    echo
done

# Build workspace root if it has binaries
log_info "Building workspace root..."
if cargo build > /dev/null 2>&1; then
    log_success "Workspace root build completed"
else
    log_warning "Workspace root build failed (this might be normal)"
fi

echo
echo "=== Build Summary ==="
echo "Projects processed: $total"
echo "Successful builds: $success"
echo "Failed builds: $failed"

if [ "$total" -eq 0 ]; then
    log_error "No projects were processed!"
    exit 1
elif [ "$failed" -gt 0 ]; then
    log_warning "Some builds failed, but script completed"
    exit 0
else
    log_success "All builds completed successfully!"
    exit 0
fi