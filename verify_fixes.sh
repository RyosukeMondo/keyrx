#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

check_command() {
    if ! command -v $1 &> /dev/null; then
        log_error "$1 could not be found. Please install it."
        exit 1
    fi
}

# Check requirements
check_command cargo
check_command flutter
check_command just

# Rust Checks
log_info "Running Rust checks..."
cd core
if cargo clippy --all-targets -- -D warnings; then
    log_info "Rust Clippy passed."
else
    log_error "Rust Clippy failed."
    exit 1
fi

if cargo test; then
    log_info "Rust Tests passed."
else
    log_error "Rust Tests failed."
    exit 1
fi
cd ..

# Dart Bindings
log_info "Regenerating Dart bindings..."
if just gen-dart-bindings > gen_bindings.log 2>&1; then
    log_info "Bindings generated successfully."
    # Move models only if generation succeeded
    if [ -f "ui/lib/models/generated_models.dart" ]; then
        mv ui/lib/models/generated_models.dart packages/core_logic/lib/models/generated_models.dart
        log_info "Moved generated models to core_logic."
    else
        log_warn "generated_models.dart not found in ui/lib/models, skipping move."
    fi
else
    log_error "Binding regeneration failed. Check gen_bindings.log for details."
    cat gen_bindings.log
    exit 1
fi

# Flutter/Dart Analysis
log_info "Running Flutter analysis..."
cd ui
if flutter analyze > ../current_analyze_output_4.txt 2>&1; then
     log_info "Flutter analysis passed."
else
     log_warn "Flutter analysis found issues. Output saved to current_analyze_output_4.txt"
     # We check the output for 'no issues found' just in case, or rely on exit code if it's strict
     if grep -q "No issues found" ../current_analyze_output_4.txt; then
        log_info "Flutter analysis passed (verified output)."
     else
        log_error "Flutter analysis failed."
        # cat ../current_analyze_output_4.txt # Optional: Print output on failure
        exit 1
     fi
fi
cd ..

log_info "Running Server analysis..."
cd apps/keyrx_server
if dart analyze > ../../server_analyze_output.txt 2>&1; then
    log_info "Server analysis passed."
else
    log_warn "Server analysis found issues. Output saved to server_analyze_output.txt"
     if grep -q "No issues found" ../../server_analyze_output.txt; then
        log_info "Server analysis passed (verified output)."
     else
        log_error "Server analysis failed."
        exit 1
     fi
fi
cd ../..

log_info "All verification steps completed."
