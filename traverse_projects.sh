#!/bin/bash
# /home/dweese/dev/rust/rabbitmq_workspace/traverse_projects.sh
# Script to clean and build all Rust projects in workspace subdirectories

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Get the current workspace directory
WORKSPACE_DIR=$(pwd)
print_status "Starting in workspace: $WORKSPACE_DIR"

# Counter for projects processed
processed_count=0
success_count=0
error_count=0

# Find all directories containing Cargo.toml files
for dir in */; do
    if [ -d "$dir" ] && [ -f "${dir}Cargo.toml" ]; then
        project_name=$(basename "$dir")
        print_status "Processing Rust project: $project_name"
        
        # Change to project directory
        cd "$dir" || {
            print_error "Failed to change directory to $dir"
            ((error_count++))
            continue
        }
        
        # Run cargo clean
        print_status "Running 'cargo clean' in $project_name..."
        if cargo clean; then
            print_success "cargo clean completed for $project_name"
        else
            print_error "cargo clean failed for $project_name"
            cd "$WORKSPACE_DIR"
            ((error_count++))
            continue
        fi
        
        # Run cargo build
        print_status "Running 'cargo build' in $project_name..."
        if cargo build; then
            print_success "cargo build completed for $project_name"
            ((success_count++))
        else
            print_error "cargo build failed for $project_name"
            ((error_count++))
        fi
        
        # Return to workspace directory
        cd "$WORKSPACE_DIR" || {
            print_error "Failed to return to workspace directory"
            exit 1
        }
        
        ((processed_count++))
        echo ""
    fi
done

# Summary
print_status "=== Build Summary ==="
print_status "Projects processed: $processed_count"
print_success "Successful builds: $success_count"
if [ $error_count -gt 0 ]; then
    print_error "Failed builds: $error_count"
else
    print_status "Failed builds: $error_count"
fi

if [ $processed_count -eq 0 ]; then
    print_warning "No Rust projects found in the current workspace!"
    print_warning "Make sure you're in the correct directory and that subdirectories contain Cargo.toml files."
    exit 1
fi

if [ $error_count -gt 0 ]; then
    exit 1
else
    print_success "All builds completed successfully!"
    exit 0
fi