#!/bin/bash
set -e

echo "Starting binding regeneration..."
just gen-dart-bindings > gen_bindings.log 2>&1

echo "Binding regeneration complete. Checking status..."
if [ $? -eq 0 ]; then
    echo "Bindings generated successfully. Moving models to core_logic..."
    mv ui/lib/models/generated_models.dart packages/core_logic/lib/models/generated_models.dart

    echo "Running flutter analyze..."
    cd ui
    flutter analyze > ../current_analyze_output_4.txt 2>&1 || true
    echo "Analysis complete. Output saved to current_analyze_output_4.txt"
else
    echo "Binding regeneration failed. Check gen_bindings.log"
    exit 1
fi
