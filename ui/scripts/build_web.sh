#!/bin/bash
# Build script for Flutter web with environment configuration

set -e

# Default values
BACKEND_URL="${KEYRX_BACKEND_URL:-http://localhost:8080}"
BASE_HREF="${FLUTTER_BASE_HREF:-/}"

# Load .env file if it exists
if [ -f "web/.env" ]; then
  echo "Loading environment from web/.env"
  set -a
  source web/.env
  set +a

  BACKEND_URL="${KEYRX_BACKEND_URL:-$BACKEND_URL}"
  BASE_HREF="${FLUTTER_BASE_HREF:-$BASE_HREF}"
fi

echo "Building Flutter web app..."
echo "  Backend URL: $BACKEND_URL"
echo "  Base href: $BASE_HREF"

# Build the web app
flutter build web \
  --release \
  --base-href="$BASE_HREF" \
  --dart-define=BACKEND_URL="$BACKEND_URL"

echo ""
echo "Build complete! Output in: build/web"
echo ""
echo "To test locally, run:"
echo "  cd build/web && python3 -m http.server 8000"
echo "Then open: http://localhost:8000"
