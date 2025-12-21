#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACT_DIR="$PROJECT_ROOT/core/src/ffi/contracts"

MAJOR_BREAKING=()
MINOR_COMPATIBLE=()
PATCH_CHANGES=()
ERRORS=()

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Detect breaking changes in FFI contracts between commits.

OPTIONS:
    -h, --help              Show this help message
    -b, --base <ref>        Base git reference to compare against (default: HEAD~1)
    -f, --format <format>   Output format: text, json, or markdown (default: text)
    --ci                    CI mode: JSON output, non-zero exit on breaking changes
    --allow-breaking        Allow breaking changes (exit 0 even if found)

EXAMPLES:
    $(basename "$0")                           # Compare against previous commit
    $(basename "$0") -b main                   # Compare against main branch
    $(basename "$0") --format json             # Output JSON format
    $(basename "$0") --ci                      # CI mode with JSON output

EXIT CODES:
    0 - No breaking changes detected (or --allow-breaking set)
    1 - Breaking changes detected without version bump
    2 - Script error
EOF
}

log_info() {
    if [[ "${FORMAT}" != "json" ]]; then
        echo "[INFO] $*" >&2
    fi
}

log_error() {
    if [[ "${FORMAT}" != "json" ]]; then
        echo "[ERROR] $*" >&2
    fi
}

classify_change() {
    local file="$1"
    local change_type="$2"
    local details="$3"

    case "$change_type" in
        "function_removed"|"param_type_changed"|"return_type_changed"|"param_removed"|"domain_changed")
            MAJOR_BREAKING+=("$file: $change_type - $details")
            ;;
        "function_added"|"param_added_optional"|"new_error_type")
            MINOR_COMPATIBLE+=("$file: $change_type - $details")
            ;;
        "description_changed"|"param_renamed"|"metadata_updated")
            PATCH_CHANGES+=("$file: $change_type - $details")
            ;;
        *)
            log_error "Unknown change type: $change_type"
            ;;
    esac
}

detect_contract_changes() {
    local base_ref="$1"
    local contract_file="$2"

    log_info "Analyzing $contract_file"

    # Get the old version of the contract
    local old_contract
    if ! old_contract=$(git show "$base_ref:$contract_file" 2>/dev/null); then
        # File didn't exist before - this is a new contract (minor change)
        local domain
        domain=$(jq -r '.domain // "unknown"' "$contract_file" 2>/dev/null || echo "unknown")
        classify_change "$contract_file" "function_added" "New contract domain: $domain"
        return
    fi

    # Check if file was deleted
    if [[ ! -f "$contract_file" ]]; then
        local domain
        domain=$(echo "$old_contract" | jq -r '.domain // "unknown"' 2>/dev/null || echo "unknown")
        classify_change "$contract_file" "function_removed" "Entire contract removed: $domain"
        return
    fi

    local new_contract
    new_contract=$(cat "$contract_file")

    # Compare domain
    local old_domain new_domain
    old_domain=$(echo "$old_contract" | jq -r '.domain // "unknown"')
    new_domain=$(echo "$new_contract" | jq -r '.domain // "unknown"')

    if [[ "$old_domain" != "$new_domain" ]]; then
        classify_change "$contract_file" "domain_changed" "$old_domain -> $new_domain"
    fi

    # Get function lists
    local old_functions new_functions
    old_functions=$(echo "$old_contract" | jq -r '.functions[]? | .name' | sort)
    new_functions=$(echo "$new_contract" | jq -r '.functions[]? | .name' | sort)

    # Detect removed functions
    while IFS= read -r func; do
        if [[ -n "$func" ]] && ! echo "$new_functions" | grep -qx "$func"; then
            classify_change "$contract_file" "function_removed" "Function removed: $func"
        fi
    done <<< "$old_functions"

    # Detect added functions
    while IFS= read -r func; do
        if [[ -n "$func" ]] && ! echo "$old_functions" | grep -qx "$func"; then
            classify_change "$contract_file" "function_added" "Function added: $func"
        fi
    done <<< "$new_functions"

    # Analyze each function that exists in both
    while IFS= read -r func; do
        if [[ -z "$func" ]]; then continue; fi

        if echo "$new_functions" | grep -qx "$func"; then
            analyze_function_changes "$old_contract" "$new_contract" "$func" "$contract_file"
        fi
    done <<< "$old_functions"
}

analyze_function_changes() {
    local old_contract="$1"
    local new_contract="$2"
    local func_name="$3"
    local contract_file="$4"

    # Get function objects
    local old_func new_func
    old_func=$(echo "$old_contract" | jq --arg name "$func_name" '.functions[] | select(.name == $name)')
    new_func=$(echo "$new_contract" | jq --arg name "$func_name" '.functions[] | select(.name == $name)')

    # Compare return types
    local old_return new_return
    old_return=$(echo "$old_func" | jq -r '.returns.type // "void"')
    new_return=$(echo "$new_func" | jq -r '.returns.type // "void"')

    if [[ "$old_return" != "$new_return" ]]; then
        classify_change "$contract_file" "return_type_changed" "$func_name: $old_return -> $new_return"
    fi

    # Compare parameters
    local old_param_count new_param_count
    old_param_count=$(echo "$old_func" | jq '.parameters | length')
    new_param_count=$(echo "$new_func" | jq '.parameters | length')

    # Get parameter names and types
    local old_params new_params
    old_params=$(echo "$old_func" | jq -r '.parameters[]? | "\(.name):\(.type):\(.required // true)"')
    new_params=$(echo "$new_func" | jq -r '.parameters[]? | "\(.name):\(.type):\(.required // true)"')

    # Check for removed parameters
    while IFS=: read -r name type required; do
        if [[ -n "$name" ]]; then
            if ! echo "$new_params" | grep -q "^$name:"; then
                classify_change "$contract_file" "param_removed" "$func_name: Parameter removed: $name"
            else
                # Check if type changed
                local new_type
                new_type=$(echo "$new_params" | grep "^$name:" | cut -d: -f2)
                if [[ "$type" != "$new_type" ]]; then
                    classify_change "$contract_file" "param_type_changed" "$func_name.$name: $type -> $new_type"
                fi

                # Check if required changed from true to false (compatible)
                local new_required
                new_required=$(echo "$new_params" | grep "^$name:" | cut -d: -f3)
                if [[ "$required" == "true" ]] && [[ "$new_required" == "false" ]]; then
                    classify_change "$contract_file" "param_added_optional" "$func_name.$name: made optional"
                elif [[ "$required" == "false" ]] && [[ "$new_required" == "true" ]]; then
                    classify_change "$contract_file" "param_type_changed" "$func_name.$name: made required (breaking)"
                fi
            fi
        fi
    done <<< "$old_params"

    # Check for added parameters
    while IFS=: read -r name type required; do
        if [[ -n "$name" ]]; then
            if ! echo "$old_params" | grep -q "^$name:"; then
                if [[ "$required" == "true" ]]; then
                    classify_change "$contract_file" "param_removed" "$func_name: Required parameter added: $name (breaking for existing calls)"
                else
                    classify_change "$contract_file" "param_added_optional" "$func_name: Optional parameter added: $name"
                fi
            fi
        fi
    done <<< "$new_params"
}

check_version_bump() {
    local base_ref="$1"

    # Get current version from Cargo.toml
    local current_version
    current_version=$(grep -m1 '^version = ' "$PROJECT_ROOT/core/Cargo.toml" | sed 's/version = "\(.*\)"/\1/' || echo "unknown")

    # Get previous version
    local previous_version
    previous_version=$(git show "$base_ref:core/Cargo.toml" 2>/dev/null | grep -m1 '^version = ' | sed 's/version = "\(.*\)"/\1/' || echo "unknown")

    if [[ "$current_version" == "$previous_version" ]]; then
        return 1
    fi

    # Check if it's a major version bump
    local current_major previous_major
    current_major=$(echo "$current_version" | cut -d. -f1)
    previous_major=$(echo "$previous_version" | cut -d. -f1)

    if [[ "$current_major" -gt "$previous_major" ]]; then
        return 0
    fi

    return 1
}

output_results() {
    local format="$1"

    case "$format" in
        json)
            output_json
            ;;
        markdown)
            output_markdown
            ;;
        *)
            output_text
            ;;
    esac
}

output_text() {
    echo ""
    echo "========================================"
    echo "FFI Breaking Change Detection Report"
    echo "========================================"
    echo ""

    if [[ ${#MAJOR_BREAKING[@]} -eq 0 ]] && [[ ${#MINOR_COMPATIBLE[@]} -eq 0 ]] && [[ ${#PATCH_CHANGES[@]} -eq 0 ]]; then
        echo "No changes detected."
        return
    fi

    if [[ ${#MAJOR_BREAKING[@]} -gt 0 ]]; then
        echo "MAJOR BREAKING CHANGES (${#MAJOR_BREAKING[@]}):"
        echo "----------------------------------------"
        printf '%s\n' "${MAJOR_BREAKING[@]}"
        echo ""
    fi

    if [[ ${#MINOR_COMPATIBLE[@]} -gt 0 ]]; then
        echo "MINOR COMPATIBLE CHANGES (${#MINOR_COMPATIBLE[@]}):"
        echo "----------------------------------------"
        printf '%s\n' "${MINOR_COMPATIBLE[@]}"
        echo ""
    fi

    if [[ ${#PATCH_CHANGES[@]} -gt 0 ]]; then
        echo "PATCH CHANGES (${#PATCH_CHANGES[@]}):"
        echo "----------------------------------------"
        printf '%s\n' "${PATCH_CHANGES[@]}"
        echo ""
    fi

    echo "========================================"
    echo "Summary:"
    echo "  Major Breaking: ${#MAJOR_BREAKING[@]}"
    echo "  Minor Compatible: ${#MINOR_COMPATIBLE[@]}"
    echo "  Patch: ${#PATCH_CHANGES[@]}"
    echo "========================================"
}

output_json() {
    local version_bumped="false"
    if check_version_bump "$BASE_REF"; then
        version_bumped="true"
    fi

    # Helper function to convert array to JSON, handling empty arrays
    to_json_array() {
        local -n arr=$1
        if [[ ${#arr[@]} -eq 0 ]]; then
            echo "[]"
        else
            printf '%s\n' "${arr[@]}" | jq -R . | jq -s .
        fi
    }

    local major_json minor_json patch_json
    major_json=$(to_json_array MAJOR_BREAKING)
    minor_json=$(to_json_array MINOR_COMPATIBLE)
    patch_json=$(to_json_array PATCH_CHANGES)

    jq -n \
        --argjson major "$major_json" \
        --argjson minor "$minor_json" \
        --argjson patch "$patch_json" \
        --arg version_bumped "$version_bumped" \
        '{
            major_breaking: $major,
            minor_compatible: $minor,
            patch_changes: $patch,
            summary: {
                major_count: ($major | length),
                minor_count: ($minor | length),
                patch_count: ($patch | length),
                version_bumped: ($version_bumped == "true")
            }
        }'
}

output_markdown() {
    echo "# FFI Breaking Change Detection Report"
    echo ""

    if [[ ${#MAJOR_BREAKING[@]} -eq 0 ]] && [[ ${#MINOR_COMPATIBLE[@]} -eq 0 ]] && [[ ${#PATCH_CHANGES[@]} -eq 0 ]]; then
        echo "No changes detected."
        return
    fi

    if [[ ${#MAJOR_BREAKING[@]} -gt 0 ]]; then
        echo "## ⚠️ MAJOR BREAKING CHANGES (${#MAJOR_BREAKING[@]})"
        echo ""
        printf '- %s\n' "${MAJOR_BREAKING[@]}"
        echo ""
    fi

    if [[ ${#MINOR_COMPATIBLE[@]} -gt 0 ]]; then
        echo "## ✨ MINOR COMPATIBLE CHANGES (${#MINOR_COMPATIBLE[@]})"
        echo ""
        printf '- %s\n' "${MINOR_COMPATIBLE[@]}"
        echo ""
    fi

    if [[ ${#PATCH_CHANGES[@]} -gt 0 ]]; then
        echo "## 🔧 PATCH CHANGES (${#PATCH_CHANGES[@]})"
        echo ""
        printf '- %s\n' "${PATCH_CHANGES[@]}"
        echo ""
    fi

    echo "## Summary"
    echo ""
    echo "| Change Type | Count |"
    echo "|-------------|-------|"
    echo "| Major Breaking | ${#MAJOR_BREAKING[@]} |"
    echo "| Minor Compatible | ${#MINOR_COMPATIBLE[@]} |"
    echo "| Patch | ${#PATCH_CHANGES[@]} |"
}

main() {
    # Parse arguments
    BASE_REF="HEAD~1"
    FORMAT="text"
    CI_MODE=false
    ALLOW_BREAKING=false

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                usage
                exit 0
                ;;
            -b|--base)
                BASE_REF="$2"
                shift 2
                ;;
            -f|--format)
                FORMAT="$2"
                shift 2
                ;;
            --ci)
                CI_MODE=true
                FORMAT="json"
                shift
                ;;
            --allow-breaking)
                ALLOW_BREAKING=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 2
                ;;
        esac
    done

    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "Not in a git repository"
        exit 2
    fi

    # Check if base ref exists
    if ! git rev-parse "$BASE_REF" > /dev/null 2>&1; then
        log_error "Base reference not found: $BASE_REF"
        exit 2
    fi

    # Check dependencies
    if ! command -v jq &> /dev/null; then
        log_error "jq is required but not installed"
        exit 2
    fi

    cd "$PROJECT_ROOT"

    log_info "Comparing against: $BASE_REF"
    log_info "Contract directory: $CONTRACT_DIR"

    # Get list of all contract files (current and previous)
    local all_contracts
    all_contracts=$(find "$CONTRACT_DIR" -name "*.ffi-contract.json" 2>/dev/null || true)
    all_contracts+=$'\n'
    all_contracts+=$(git ls-tree -r --name-only "$BASE_REF" "$CONTRACT_DIR" 2>/dev/null | grep "\.ffi-contract\.json$" || true)
    all_contracts=$(echo "$all_contracts" | sort -u | grep -v "^$")

    # Analyze each contract
    while IFS= read -r contract; do
        if [[ -n "$contract" ]]; then
            detect_contract_changes "$BASE_REF" "$contract"
        fi
    done <<< "$all_contracts"

    # Output results
    output_results "$FORMAT"

    # Determine exit code
    if [[ ${#MAJOR_BREAKING[@]} -gt 0 ]]; then
        if [[ "$ALLOW_BREAKING" == true ]]; then
            exit 0
        fi

        if check_version_bump "$BASE_REF"; then
            log_info "Breaking changes detected, but version was bumped"
            exit 0
        fi

        if [[ "$CI_MODE" == true ]]; then
            log_error "Breaking changes detected without version bump"
        fi
        exit 1
    fi

    exit 0
}

main "$@"
