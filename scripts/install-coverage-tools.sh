#!/usr/bin/env bash
# Install coverage tooling for Flutter/Dart: dart coverage package and lcov CLI.
# This script prefers native package managers for lcov and uses dart pub global for coverage.

set -euo pipefail

log() {
    echo "[coverage-tools] $*"
}

install_dart_coverage() {
    if command -v dart >/dev/null 2>&1; then
        log "Activating dart coverage package..."
        dart pub global activate coverage >/dev/null
        log "dart coverage activated."
    elif command -v flutter >/dev/null 2>&1; then
        log "Activating coverage via flutter pub..."
        flutter pub global activate coverage >/dev/null
        log "flutter coverage activated."
    else
        log "Dart/Flutter not found in PATH; skip coverage activation."
    fi
}

install_lcov() {
    if command -v lcov >/dev/null 2>&1; then
        log "lcov already installed."
        return
    fi

    if command -v apt-get >/dev/null 2>&1; then
        log "Installing lcov via apt-get..."
        sudo apt-get update -y >/dev/null
        sudo apt-get install -y lcov >/dev/null
    elif command -v brew >/dev/null 2>&1; then
        log "Installing lcov via Homebrew..."
        brew install lcov
    else
        log "Could not find a supported package manager to install lcov."
        exit 1
    fi

    log "lcov installed."
}

install_dart_coverage
install_lcov
log "Coverage tooling installation complete."
