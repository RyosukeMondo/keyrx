#!/usr/bin/env bash
# Flutter Linux launcher with DISPLAY set
set -e

export DISPLAY=:1

# Run flutter with all arguments passed to this script
flutter run -d linux "$@"
