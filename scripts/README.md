# Scripts Directory

This directory contains utility scripts for building, testing, and setting up the keyrx project.

## Setup Scripts

### `flutter_setup.sh`

Modular Flutter/FVM setup script for CI/CD environments.

**Prerequisites:**
- Android SDK installed and `ANDROID_HOME` environment variable set
- Git repository initialized
- Internet connection for downloading FVM and Flutter

**What it does:**
1. Installs FVM (Flutter Version Manager)
2. Installs Flutter stable version (reads from `.fvmrc`)
3. Configures project to use FVM-managed Flutter
4. Configures Flutter with Android SDK
5. Verifies setup with `flutter doctor`
6. Cleans up FVM-generated git modifications

**Usage:**
```bash
# Standalone
export ANDROID_HOME=/path/to/android-sdk
bash scripts/flutter_setup.sh

# From another script
bash scripts/flutter_setup.sh
```

**Exit codes:**
- `0`: Success
- `1`: ANDROID_HOME not set

### `jules_setup.sh`

Complete Jules CI setup script for Android/Flutter build environment.

**What it does:**
1. Installs system dependencies (Java 17, wget, unzip, curl, git)
2. Downloads and configures Android SDK (API 34, Build Tools 34.0.0)
3. Runs modular Flutter/FVM setup via `flutter_setup.sh`

**Usage:**
```bash
# In Jules CI environment
bash scripts/jules_setup.sh
```

**Note:** This script expects to run in `/app` directory (Jules CI default).

## Build Scripts

### `build.sh`
Main build script for creating release artifacts.

### `flutter_prebuild.sh`
Pre-build script for Flutter-specific preparation steps.

## Development Scripts

### `install-hooks.sh`
Installs git pre-commit hooks for code quality checks.

### `setup-linux-input.sh`
Sets up Linux input device permissions for development.

### `test_feature_combinations.sh`
Tests various Cargo feature combinations to ensure compatibility.

## Utility Scripts

### `generate-error-docs.sh`
Generates documentation from error code definitions.

### `release.sh`
Automates the release process.

### `show_key_position.sh`
Debugging tool for keyboard input positions.
