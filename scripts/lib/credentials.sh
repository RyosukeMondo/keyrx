#!/usr/bin/env bash
# Credential and Secret Handling (SSOT)
#
# SECURITY RULES:
# 1. NEVER log credentials, API keys, tokens, or secrets
# 2. NEVER print credentials to stdout/stderr
# 3. NEVER store credentials in files (use env vars only)
# 4. NEVER pass credentials as command-line arguments
# 5. ALWAYS validate credential format before use
# 6. ALWAYS use secure environment variables
#
# Usage:
#   source scripts/lib/credentials.sh
#   validate_env_var "API_KEY"
#   get_credential "API_KEY"

set -euo pipefail

# Color codes for secure output (no credential leakage)
readonly CRED_ERROR='\033[0;31m'
readonly CRED_WARN='\033[0;33m'
readonly CRED_INFO='\033[0;36m'
readonly CRED_RESET='\033[0m'

# Secure logging - NEVER logs credential values
cred_error() {
    echo -e "${CRED_ERROR}[CREDENTIAL ERROR]${CRED_RESET} $*" >&2
}

cred_warn() {
    echo -e "${CRED_WARN}[CREDENTIAL WARN]${CRED_RESET} $*" >&2
}

cred_info() {
    echo -e "${CRED_INFO}[CREDENTIAL INFO]${CRED_RESET} $*"
}

# Validate that an environment variable exists and is non-empty
# Args:
#   $1: Variable name
# Returns:
#   0 if valid, 1 if invalid
# Security: NEVER logs the value
validate_env_var() {
    local var_name="$1"

    if [ -z "${!var_name:-}" ]; then
        cred_error "Environment variable '$var_name' is not set or empty"
        cred_info "Set it with: export $var_name='your-value'"
        return 1
    fi

    return 0
}

# Get credential from environment variable
# Args:
#   $1: Variable name
# Returns:
#   Credential value (to stdout)
# Security: Only returns value via stdout, never logs
get_credential() {
    local var_name="$1"

    if ! validate_env_var "$var_name"; then
        return 1
    fi

    # Return value directly (caller should capture in variable)
    echo "${!var_name}"
}

# Validate API key format (generic)
# Args:
#   $1: API key
# Returns:
#   0 if valid format, 1 if invalid
# Security: NEVER logs the key value
validate_api_key_format() {
    local key="$1"

    # Check minimum length (most API keys are at least 20 chars)
    if [ ${#key} -lt 20 ]; then
        cred_error "API key format invalid: too short (min 20 chars)"
        return 1
    fi

    # Check contains only valid characters (alphanumeric, dash, underscore)
    if ! echo "$key" | grep -qE '^[A-Za-z0-9_-]+$'; then
        cred_error "API key format invalid: contains invalid characters"
        return 1
    fi

    return 0
}

# Validate GitHub token format
# Args:
#   $1: GitHub token
# Returns:
#   0 if valid format, 1 if invalid
# Security: NEVER logs the token value
validate_github_token() {
    local token="$1"

    # GitHub tokens start with specific prefixes
    if ! echo "$token" | grep -qE '^(ghp_|gho_|ghu_|ghs_|ghr_)'; then
        cred_error "GitHub token format invalid: must start with ghp_, gho_, ghu_, ghs_, or ghr_"
        return 1
    fi

    # GitHub tokens are 40+ characters
    if [ ${#token} -lt 40 ]; then
        cred_error "GitHub token format invalid: too short"
        return 1
    fi

    return 0
}

# Validate JWT token format
# Args:
#   $1: JWT token
# Returns:
#   0 if valid format, 1 if invalid
# Security: NEVER logs the token value
validate_jwt_token() {
    local token="$1"

    # JWT has 3 parts separated by dots
    local parts
    parts=$(echo "$token" | tr '.' '\n' | wc -l)

    if [ "$parts" -ne 3 ]; then
        cred_error "JWT token format invalid: must have 3 parts (header.payload.signature)"
        return 1
    fi

    return 0
}

# Check if running in secure context (not debugging/verbose mode)
# Returns:
#   0 if secure, 1 if potentially insecure
is_secure_context() {
    # Check for debug/verbose flags that might leak credentials
    if [ "${DEBUG:-}" = "true" ] || [ "${VERBOSE:-}" = "true" ]; then
        cred_warn "Running in DEBUG/VERBOSE mode - credentials may be exposed"
        return 1
    fi

    # Check if bash debugging is enabled
    if [[ "$-" == *x* ]]; then
        cred_warn "Bash debugging (set -x) is enabled - credentials may be exposed"
        return 1
    fi

    return 0
}

# Securely load credentials from .env file (if it exists)
# Args:
#   $1: Path to .env file (optional, defaults to .env)
# Security:
#   - NEVER logs credential values
#   - Validates .env file permissions
#   - Only loads variables that don't exist
load_env_file() {
    local env_file="${1:-.env}"

    if [ ! -f "$env_file" ]; then
        cred_info "No .env file found at: $env_file (this is OK)"
        return 0
    fi

    # Check file permissions (should not be world-readable)
    local perms
    perms=$(stat -c '%a' "$env_file" 2>/dev/null || stat -f '%A' "$env_file" 2>/dev/null || echo "")

    if [ -n "$perms" ] && [ "$perms" -gt 600 ]; then
        cred_warn ".env file has permissive permissions: $perms"
        cred_warn "Consider: chmod 600 $env_file"
    fi

    # Load variables (only if not already set)
    while IFS='=' read -r key value; do
        # Skip comments and empty lines
        if [[ "$key" =~ ^#.*$ ]] || [ -z "$key" ]; then
            continue
        fi

        # Only set if not already in environment
        if [ -z "${!key:-}" ]; then
            export "$key=$value"
        fi
    done < "$env_file"

    cred_info "Loaded credentials from: $env_file"
}

# Validate all required credentials for KeyRx daemon
# Returns:
#   0 if all valid, 1 if any missing/invalid
validate_keyrx_credentials() {
    local all_valid=true

    # Check for required environment variables
    # (Add more as needed)

    # Example: API authentication
    if [ -n "${KEYRX_API_KEY:-}" ]; then
        if ! validate_api_key_format "$KEYRX_API_KEY"; then
            all_valid=false
        fi
    fi

    # Example: GitHub integration
    if [ -n "${GITHUB_TOKEN:-}" ]; then
        if ! validate_github_token "$GITHUB_TOKEN"; then
            all_valid=false
        fi
    fi

    if [ "$all_valid" = true ]; then
        cred_info "All KeyRx credentials validated successfully"
        return 0
    else
        cred_error "Some KeyRx credentials failed validation"
        return 1
    fi
}

# Mask credential for display (show first/last chars only)
# Args:
#   $1: Credential value
# Returns:
#   Masked string
# Security: Safe to log the output
mask_credential() {
    local cred="$1"
    local length=${#cred}

    if [ "$length" -le 8 ]; then
        echo "****"
    else
        local first="${cred:0:4}"
        local last="${cred: -4}"
        echo "${first}...${last}"
    fi
}

# Securely clear credential from variable
# Args:
#   $1: Variable name
clear_credential() {
    local var_name="$1"

    if [ -n "${!var_name:-}" ]; then
        unset "$var_name"
    fi
}

# Example usage function (for documentation)
_credentials_usage_example() {
    cat <<'EOF'
Example usage:

# Load .env file securely
source scripts/lib/credentials.sh
load_env_file

# Validate specific credential
if validate_env_var "API_KEY"; then
    api_key=$(get_credential "API_KEY")
    # Use $api_key here
    clear_credential "api_key"
fi

# Validate all KeyRx credentials
if ! validate_keyrx_credentials; then
    echo "Credential validation failed"
    exit 1
fi

# Check secure context
if ! is_secure_context; then
    echo "Warning: running in potentially insecure context"
fi
EOF
}

# Self-test (only runs if script is executed directly)
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    echo "Running credentials.sh self-test..."

    # Test 1: Validate missing env var
    if validate_env_var "NONEXISTENT_VAR" 2>/dev/null; then
        echo "FAIL: Should have failed for missing var"
        exit 1
    fi

    # Test 2: Validate API key format
    if validate_api_key_format "short"; then
        echo "FAIL: Should have failed for short key"
        exit 1
    fi

    if ! validate_api_key_format "abcdefghij0123456789klmnop"; then
        echo "FAIL: Should have passed for valid key"
        exit 1
    fi

    # Test 3: Mask credential
    masked=$(mask_credential "abcdefghij0123456789")
    if [ "$masked" != "abcd...6789" ]; then
        echo "FAIL: Masking failed, got: $masked"
        exit 1
    fi

    # Test 4: Secure context check
    DEBUG=true is_secure_context 2>/dev/null && {
        echo "FAIL: Should have detected insecure context"
        exit 1
    }

    echo "All self-tests passed!"
    echo ""
    _credentials_usage_example
fi
