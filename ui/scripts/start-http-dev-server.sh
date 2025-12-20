#!/bin/bash
# Simple HTTP Development Server for Flutter Web
# Use this for quick local testing (no service worker support)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build/web"
PORT="${PORT:-4200}"
BIND_IP="${BIND_IP:-0.0.0.0}"

echo "🌐 KeyRx HTTP Development Server"
echo "================================="
echo ""

# Stop any existing servers on this port
echo "🧹 Checking for existing servers on port $PORT..."
pkill -f "python.*http.server $PORT" 2>/dev/null || true
sleep 1

# Check if build exists
if [ ! -d "$BUILD_DIR" ]; then
    echo "❌ Build directory not found: $BUILD_DIR"
    echo "Run 'flutter build web' first"
    exit 1
fi

echo "📁 Serving: $BUILD_DIR"
echo "🌐 Address: http://$BIND_IP:$PORT"
echo "🔗 Access via:"
echo "   - http://localhost:$PORT"
echo "   - http://192.168.11.24:$PORT"
echo "   - http://192.168.11.53:$PORT"
echo ""
echo "ℹ️  Note: Service workers require HTTPS"
echo "   Use start-https-dev-server.sh for service worker testing"
echo ""
echo "🛑 Press Ctrl+C to stop"
echo ""

cd "$BUILD_DIR"
python3 -m http.server "$PORT" --bind "$BIND_IP"
