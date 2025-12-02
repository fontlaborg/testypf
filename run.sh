#!/bin/bash

# TestYPF Run Script
# Builds and runs the testypf GUI application in release mode

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the testypf root directory."
    exit 1
fi

BINARY_NAME="testypf"
BINARY_PATH="target/release/$BINARY_NAME"

# Check if binary exists and is up to date
needs_build() {
    if [ ! -f "$BINARY_PATH" ]; then
        return 0
    fi
    
    # Check if Cargo.toml is newer than the binary
    if [ "Cargo.toml" -nt "$BINARY_PATH" ]; then
        return 0
    fi
    
    # Check if any source files are newer than the binary
    if find crates -name "*.rs" -newer "$BINARY_PATH" 2>/dev/null | grep -q .; then
        return 0
    fi
    
    return 1
}

# Build if needed
if needs_build; then
    print_status "Building testypf in release mode..."
    
    # Use the existing build script for release mode
    if ./build.sh --release --gui --skip-deps; then
        print_status "Build completed successfully"
    else
        print_error "Build failed"
        exit 1
    fi
else
    print_status "Binary is up to date, skipping build"
fi

# Run the application
if [ -f "$BINARY_PATH" ]; then
    print_status "Running $BINARY_NAME..."
    exec "$BINARY_PATH" "$@"
else
    print_error "Binary not found at $BINARY_PATH after build"
    exit 1
fi
