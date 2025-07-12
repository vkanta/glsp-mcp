#!/bin/bash

# Fix all export macro invocations in component files

echo "Fixing export macro invocations in all components..."

# Function to fix a single file
fix_export() {
    local file=$1
    local binding_name=$(basename $(dirname $file) | tr '-' '_')_ecu_bindings
    
    # Special cases
    case $(dirname $file) in
        *"/object-detection")
            binding_name="object_detection_ai_bindings"
            ;;
        *"/behavior-prediction")
            binding_name="behavior_prediction_ai_bindings"
            ;;
        *"/orchestrator")
            binding_name="adas_orchestrator_ecu_bindings"
            ;;
        *"/adas-visualizer")
            binding_name="adas_visualizer_ecu_bindings"
            ;;
    esac
    
    echo "Fixing $file with binding name: $binding_name"
    
    # Fix the export line
    sed -i '' "s/${binding_name}::export!(Component);/${binding_name}::export!(Component with_types_in ${binding_name});/g" "$file"
}

# Find all component lib.rs files
find components -name "lib.rs" -path "*/src/*" | while read file; do
    fix_export "$file"
done

echo "All export macros have been fixed!"