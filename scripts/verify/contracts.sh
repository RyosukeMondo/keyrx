#!/usr/bin/env bash
#
# FFI Contract Validation Script
#
# This script validates that all #[ffi_export] annotated functions have matching
# contract JSON entries and that contracts match Rust signatures.
#
# Usage:
#   ./scripts/validate_ffi_contracts.sh          # Validate only
#   ./scripts/validate_ffi_contracts.sh --fix    # Auto-regenerate contracts
#   ./scripts/validate_ffi_contracts.sh --check  # CI mode (strict validation)
#
# Exit codes:
#   0 - All contracts valid
#   1 - Validation errors found
#   2 - Script error (missing dependencies, etc.)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Find repository root
find_repo_root() {
    local dir="$PWD"
    while [[ "$dir" != "/" ]]; do
        if [[ -d "$dir/.git" ]]; then
            echo "$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    return 1
}

REPO_ROOT="$(find_repo_root)"
if [[ -z "$REPO_ROOT" ]]; then
    echo -e "${RED}Error: Could not find repository root${NC}" >&2
    exit 2
fi

CORE_DIR="$REPO_ROOT/core"
FFI_DIR="$CORE_DIR/src/ffi"
CONTRACTS_DIR="$FFI_DIR/contracts"
TEMP_DIR="$REPO_ROOT/.cache/contract-validation"

# Parse command line arguments
FIX_MODE=false
CHECK_MODE=false
VERBOSE=false

for arg in "$@"; do
    case $arg in
        --fix)
            FIX_MODE=true
            ;;
        --check)
            CHECK_MODE=true
            ;;
        --verbose)
            VERBOSE=true
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --fix       Auto-regenerate contracts for mismatches"
            echo "  --check     CI mode - strict validation, no auto-fix"
            echo "  --verbose   Show detailed validation output"
            echo "  --help      Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $arg${NC}" >&2
            echo "Use --help for usage information"
            exit 2
            ;;
    esac
done

# Create temp directory
mkdir -p "$TEMP_DIR"

# Step 1: Extract all #[ffi_export] annotations from Rust code
echo -e "${BLUE}[1/4]${NC} Extracting FFI export annotations from Rust code..."

# Find all Rust files with #[ffi_export] annotations
FFI_EXPORT_FILES=$(grep -r -l "#\[ffi_export" "$FFI_DIR" --include="*.rs" 2>/dev/null || true)

if [[ -z "$FFI_EXPORT_FILES" ]]; then
    echo -e "${YELLOW}Warning: No #[ffi_export] annotations found${NC}"
    echo -e "${GREEN}✓ Validation complete (no FFI exports to validate)${NC}"
    exit 0
fi

# Count total functions with #[ffi_export]
TOTAL_EXPORTS=$(echo "$FFI_EXPORT_FILES" | xargs grep -h "#\[ffi_export" 2>/dev/null | wc -l)
echo -e "  Found ${GREEN}$TOTAL_EXPORTS${NC} #[ffi_export] annotations"

# Step 2: Extract function names and domains using Python
echo -e "${BLUE}[2/4]${NC} Parsing function signatures and domains..."

EXPECTED_CONTRACTS="$TEMP_DIR/expected_contracts.txt"

# Use Python to parse the Rust files
python3 - << 'EOF' > "$EXPECTED_CONTRACTS"
import re
import sys
import os

ffi_dir = os.environ.get('FFI_DIR', '')
verbose = os.environ.get('VERBOSE', 'false') == 'true'
repo_root = os.environ.get('REPO_ROOT', '')

# Find all Rust files with #[ffi_export]
for root, dirs, files in os.walk(ffi_dir):
    for file in files:
        if not file.endswith('.rs'):
            continue

        filepath = os.path.join(root, file)

        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            if verbose:
                print(f"Warning: Could not read {filepath}: {e}", file=sys.stderr)
            continue

        # Find all #[ffi_export(...)] annotations and their following functions
        # Pattern: #[ffi_export(domain = "...")] followed by pub fn ...
        pattern = r'#\[ffi_export\([^)]*domain\s*=\s*"([^"]+)"[^)]*\)\][^{]*?(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)'

        matches = re.finditer(pattern, content, re.MULTILINE | re.DOTALL)

        for match in matches:
            domain = match.group(1)
            fn_name = match.group(2)

            if verbose:
                rel_path = os.path.relpath(filepath, repo_root)
                print(f"  Found: {domain}.{fn_name} in {rel_path}", file=sys.stderr)

            print(f"{domain}:{fn_name}:{filepath}")

EOF

if [[ $? -ne 0 ]]; then
    echo -e "${RED}Error: Failed to parse Rust files${NC}"
    exit 2
fi

EXPECTED_COUNT=$(wc -l < "$EXPECTED_CONTRACTS")
echo -e "  Parsed ${GREEN}$EXPECTED_COUNT${NC} FFI export functions"

if [[ $EXPECTED_COUNT -eq 0 ]]; then
    echo -e "${YELLOW}Warning: No #[ffi_export] functions parsed${NC}"
    echo -e "  This might indicate a parsing issue"
fi

# Step 3: Verify all expected contracts exist in JSON files
echo -e "${BLUE}[3/4]${NC} Validating contract JSON files..."

MISSING_CONTRACTS=0
MISMATCHED_CONTRACTS=0
VALID_CONTRACTS=0

# Read expected contracts and check JSON files
while IFS=: read -r domain fn_name source_file; do
    CONTRACT_FILE="$CONTRACTS_DIR/${domain}.ffi-contract.json"

    if [[ ! -f "$CONTRACT_FILE" ]]; then
        echo -e "${RED}  ✗ Missing contract file: ${domain}.ffi-contract.json${NC}"
        echo -e "    Function: $fn_name (from ${source_file#$REPO_ROOT/})"
        MISSING_CONTRACTS=$((MISSING_CONTRACTS + 1))
        continue
    fi

    # Check if function exists in contract JSON
    if ! grep -q "\"name\"[[:space:]]*:[[:space:]]*\"$fn_name\"" "$CONTRACT_FILE" 2>/dev/null; then
        echo -e "${RED}  ✗ Missing contract entry: $domain.$fn_name${NC}"
        echo -e "    Expected in: ${CONTRACT_FILE#$REPO_ROOT/}"
        echo -e "    Source: ${source_file#$REPO_ROOT/}"
        MISMATCHED_CONTRACTS=$((MISMATCHED_CONTRACTS + 1))
    else
        VALID_CONTRACTS=$((VALID_CONTRACTS + 1))
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "${GREEN}  ✓ $domain.$fn_name${NC}"
        fi
    fi
done < "$EXPECTED_CONTRACTS"

# Step 4: Check for orphaned contracts (in JSON but not in Rust)
echo -e "${BLUE}[4/4]${NC} Checking for orphaned contract entries..."

ORPHANED_CONTRACTS=0

for contract_file in "$CONTRACTS_DIR"/*.ffi-contract.json; do
    if [[ ! -f "$contract_file" ]]; then
        continue
    fi

    # Extract domain from filename
    domain=$(basename "$contract_file" .ffi-contract.json)

    # Extract all function names from JSON using Python
    json_functions=$(python3 - "$contract_file" << 'PYEOF'
import json
import sys

try:
    with open(sys.argv[1], 'r') as f:
        data = json.load(f)
        for func in data.get('functions', []):
            print(func.get('name', ''))
except Exception as e:
    sys.exit(1)
PYEOF
)

    # Check each JSON function against expected contracts
    for json_fn in $json_functions; do
        if ! grep -q "^$domain:$json_fn:" "$EXPECTED_CONTRACTS" 2>/dev/null; then
            echo -e "${YELLOW}  ! Orphaned contract: $domain.$json_fn${NC}"
            echo -e "    Found in: ${contract_file#$REPO_ROOT/}"
            echo -e "    No matching #[ffi_export] annotation in Rust code"
            ORPHANED_CONTRACTS=$((ORPHANED_CONTRACTS + 1))
        fi
    done
done

# Summary
echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║           FFI Contract Validation Summary                  ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo -e "  Valid contracts:     ${GREEN}$VALID_CONTRACTS${NC}"
echo -e "  Missing contracts:   ${RED}$MISSING_CONTRACTS${NC}"
echo -e "  Mismatched:          ${RED}$MISMATCHED_CONTRACTS${NC}"
echo -e "  Orphaned:            ${YELLOW}$ORPHANED_CONTRACTS${NC}"
echo ""

TOTAL_ISSUES=$((MISSING_CONTRACTS + MISMATCHED_CONTRACTS))

if [[ $TOTAL_ISSUES -eq 0 ]]; then
    if [[ $ORPHANED_CONTRACTS -gt 0 ]]; then
        echo -e "${YELLOW}⚠ Validation passed with warnings${NC}"
        echo -e "  $ORPHANED_CONTRACTS orphaned contract(s) found"
        echo -e "  These contracts exist in JSON but have no matching Rust code"
        echo ""
        if [[ "$CHECK_MODE" == "true" ]]; then
            echo -e "${RED}✗ CI mode: Orphaned contracts are not allowed${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}✓ All contracts are valid!${NC}"
    fi
    exit 0
fi

# Handle errors
echo -e "${RED}✗ Validation failed with $TOTAL_ISSUES error(s)${NC}"
echo ""

if [[ "$FIX_MODE" == "true" ]]; then
    echo -e "${BLUE}Attempting to auto-fix by regenerating contracts...${NC}"
    echo ""

    # Run the contract generation tool
    cd "$REPO_ROOT"
    if just gen-dart-bindings; then
        echo ""
        echo -e "${GREEN}✓ Contracts regenerated successfully${NC}"
        echo -e "  Run this script again to verify the fixes"
        exit 0
    else
        echo ""
        echo -e "${RED}✗ Failed to regenerate contracts${NC}"
        exit 1
    fi
else
    echo "To fix these issues:"
    echo "  1. Run with --fix flag to auto-regenerate contracts:"
    echo "     $0 --fix"
    echo ""
    echo "  2. Or manually regenerate contracts:"
    echo "     just gen-dart-bindings"
    echo ""
    echo "  3. Ensure all #[ffi_export] functions are correct"
    echo ""
fi

exit 1
