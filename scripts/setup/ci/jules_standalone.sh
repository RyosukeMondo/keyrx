#!/bin/bash
set -e

# ========================================================================================
# Jules Standalone Setup Script
# ========================================================================================
# This script is designed to be COPIED AND PASTED directly into the Jules "Initial Setup"
# configuration block. It contains all necessary setup logic (Android SDK + Flutter)
# in a single file, removing dependencies on repository files being present immediately.
#
# Optimization:
# - Relies on pre-installed Java 21, git, curl in Jules VM.
# - Inlines Flutter/FVM setup logic.
# ========================================================================================

echo "Starting Jules Android/Flutter Environment Setup (Standalone)..."

# Ensure we are in the app directory (Jules default)
cd /app

# --- 1. SYSTEM DEPENDENCIES ---
echo "Installing system dependencies..."
sudo apt-get update
# Java 21, git, curl are pre-installed. Ensure unzip/wget for SDK tools.
sudo apt-get install -y unzip wget

# --- 2. ANDROID SDK SETUP ---
echo "Setting up Android SDK..."

export ANDROID_HOME=$HOME/android-sdk
export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools

# Create directory structure
mkdir -p $ANDROID_HOME/cmdline-tools

# Download Command Line Tools
echo "Downloading Android Command Line Tools..."
wget -q https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip -O cmdline-tools.zip

# Unzip and restructure
unzip -q cmdline-tools.zip -d $ANDROID_HOME/cmdline-tools
mv $ANDROID_HOME/cmdline-tools/cmdline-tools $ANDROID_HOME/cmdline-tools/latest
rm cmdline-tools.zip

# Install Platforms and Build Tools
echo "Installing Android SDK Platforms and Tools..."
# Accept licenses
yes | sdkmanager --licenses > /dev/null
# Install specific versions needed for KeyRx
sdkmanager "platform-tools" "platforms;android-34" "build-tools;34.0.0"

# --- 3. FLUTTER & FVM SETUP (Inlined) ---
echo "Starting Flutter/FVM Setup..."

# Install FVM
echo "Installing FVM..."
curl -fsSL https://fvm.app/install.sh | bash

# Add FVM to PATH
export PATH="$HOME/.fvm_flutter/bin:$PATH"

# Install Flutter
echo "Installing Flutter stable..."
fvm install stable

# Configure project
echo "Configuring project to use FVM Flutter..."
fvm use stable --force

# Configure Android SDK in Flutter
echo "Configuring Flutter with Android SDK..."
fvm flutter config --android-sdk "$ANDROID_HOME"

# Verify
echo "Verifying Flutter setup..."
fvm flutter doctor -v

# Cleanup
echo "Cleaning up FVM-generated git changes..."
# If the repo is present, reset changes. If not (running before clone?), this might fail or do nothing.
if [ -d .git ]; then
    git reset --hard HEAD
    git clean -fd
fi

# --- 4. DONE ---
echo "Environment Setup Complete. Ready to build."
