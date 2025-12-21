#!/bin/bash
# Verify consistency between CLI and persisted device registry.
set -e

# Use a temporary directory for configuration to avoid messing with user settings
CONFIG_ROOT="$(pwd)/tmp_registry_test"
rm -rf "$CONFIG_ROOT"
mkdir -p "$CONFIG_ROOT"

echo "Step 1: Adding virtual device via CLI..."
# 0x1234 = 4660, 0x5678 = 22136
echo '{"command": "addVirtual", "payload": {"vendor_id": 4660, "product_id": 22136, "serial_number": "VIRTUAL-TEST-123", "user_label": "Test Device"}, "id": 1}' | \
  dart run apps/cli_tool/bin/keyrx_cli.dart --config-root "$CONFIG_ROOT" --lib-path ./target/debug/libkeyrx_core.so > /dev/null

echo "Step 2: Verifying persistence in devices.json..."
if [ ! -f "$CONFIG_ROOT/devices.json" ]; then
    echo "❌ Error: devices.json was not created."
    exit 1
fi

if grep -q "VIRTUAL-TEST-123" "$CONFIG_ROOT/devices.json"; then
    echo "✅ Success: Device found in config file."
else
    echo "❌ Failure: Device not found in config file."
    cat "$CONFIG_ROOT/devices.json"
    exit 1
fi

echo "Step 3: Setting user label via CLI..."
# Key format is vid:pid:serial (hex vid/pid)
echo '{"command": "setLabel", "payload": {"key": "1234:5678:VIRTUAL-TEST-123", "label": "Updated Label"}, "id": 2}' | \
  dart run apps/cli_tool/bin/keyrx_cli.dart --config-root "$CONFIG_ROOT" --lib-path ./target/debug/libkeyrx_core.so > /dev/null

echo "Step 4: Verifying label update..."
if grep -q "Updated Label" "$CONFIG_ROOT/devices.json"; then
    echo "✅ Success: Label updated in config file."
else
    echo "❌ Failure: Label not updated in config file."
    cat "$CONFIG_ROOT/devices.json"
    exit 1
fi

echo "Step 5: Listing devices via CLI..."
LIST_OUTPUT=$(echo '{"command": "listDevices", "id": 3}' | \
  dart run apps/cli_tool/bin/keyrx_cli.dart --config-root "$CONFIG_ROOT" --lib-path ./target/debug/libkeyrx_core.so)

if echo "$LIST_OUTPUT" | grep -q "Updated Label"; then
    echo "✅ Success: Updated label reflected in listDevices output."
else
    echo "❌ Failure: Updated label missing from listDevices output."
    echo "$LIST_OUTPUT"
    exit 1
fi

# Clean up
rm -rf "$CONFIG_ROOT"
echo "🎉 All consistency checks passed!"
