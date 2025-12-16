#!/usr/bin/env python3
"""
Test client for KeyRx daemon IPC interface.

This script connects to the daemon's Unix socket and sends JSON-RPC commands.
"""

import socket
import json
import sys
import os
import time

DEFAULT_SOCKET = os.path.expandvars("${XDG_RUNTIME_DIR}/keyrx/daemon.sock")
if not os.path.exists(os.path.dirname(DEFAULT_SOCKET)):
    DEFAULT_SOCKET = "/tmp/keyrx/daemon.sock"


class DaemonClient:
    def __init__(self, socket_path=DEFAULT_SOCKET):
        self.socket_path = socket_path
        self.sock = None
        self.request_id = 0

    def connect(self):
        """Connect to the daemon socket."""
        print(f"Connecting to {self.socket_path}...")
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.socket_path)
        print("Connected!")

    def send_request(self, method, params=None):
        """Send a JSON-RPC request and return the response."""
        self.request_id += 1
        request = {
            "jsonrpc": "2.0",
            "method": method,
            "id": self.request_id,
        }
        if params:
            request["params"] = params

        # Send request
        request_json = json.dumps(request) + "\n"
        print(f"\n→ Sending: {request_json.strip()}")
        self.sock.sendall(request_json.encode("utf-8"))

        # Receive response
        response = b""
        while True:
            chunk = self.sock.recv(4096)
            if not chunk:
                break
            response += chunk
            if b"\n" in response:
                break

        response_str = response.decode("utf-8").strip()
        print(f"← Received: {response_str}")

        return json.loads(response_str)

    def close(self):
        """Close the connection."""
        if self.sock:
            self.sock.close()


def main():
    if len(sys.argv) > 1:
        socket_path = sys.argv[1]
    else:
        socket_path = DEFAULT_SOCKET

    client = DaemonClient(socket_path)

    try:
        client.connect()

        # Test 1: Get status (should be "stopped" initially)
        print("\n=== Test 1: Get Status ===")
        response = client.send_request("engine.status")
        if "result" in response:
            print(f"✓ Status: {response['result']['status']}")
        else:
            print(f"✗ Error: {response.get('error')}")

        # Test 2: Start engine
        print("\n=== Test 2: Start Engine ===")
        response = client.send_request("engine.start", {
            "use_all_devices": True
        })
        if "result" in response:
            print(f"✓ Engine started")
            print(f"  Devices: {len(response['result'].get('devices', []))}")
            for device in response['result'].get('devices', []):
                print(f"    - {device['name']} ({device['path']})")
        else:
            print(f"✗ Error: {response.get('error')}")

        # Test 3: Get status again (should be "running")
        print("\n=== Test 3: Get Status (After Start) ===")
        response = client.send_request("engine.status")
        if "result" in response:
            status = response['result']
            print(f"✓ Status: {status['status']}")
            print(f"  Uptime: {status.get('uptime_seconds', 0)}s")
            print(f"  Events processed: {status.get('events_processed', 0)}")
        else:
            print(f"✗ Error: {response.get('error')}")

        # Test 4: Subscribe to events
        print("\n=== Test 4: Subscribe to Events ===")
        response = client.send_request("events.subscribe", {
            "event_types": ["input", "output", "state_change"]
        })
        if "result" in response:
            print(f"✓ Subscribed to events")
            print("  (Note: Events will be streamed as notifications)")
        else:
            print(f"✗ Error: {response.get('error')}")

        # Wait a bit for any events (optional)
        print("\n  Waiting 2 seconds for events...")
        time.sleep(2)

        # Test 5: Stop engine
        print("\n=== Test 5: Stop Engine ===")
        response = client.send_request("engine.stop")
        if "result" in response:
            print(f"✓ Engine stopped")
        else:
            print(f"✗ Error: {response.get('error')}")

        # Test 6: Get status one more time (should be "stopped")
        print("\n=== Test 6: Final Status ===")
        response = client.send_request("engine.status")
        if "result" in response:
            print(f"✓ Status: {response['result']['status']}")
        else:
            print(f"✗ Error: {response.get('error')}")

        print("\n=== All Tests Complete ===")

    except ConnectionRefusedError:
        print(f"✗ Could not connect to daemon at {socket_path}")
        print("  Make sure the daemon is running: keyrx daemon")
        sys.exit(1)
    except Exception as e:
        print(f"✗ Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        client.close()


if __name__ == "__main__":
    main()
