#!/bin/bash
# Fix component dependencies to use workspace versions

set -e

echo "🔧 Updating component dependencies to use workspace versions..."

# Function to update a component's Cargo.toml
update_component_deps() {
    local file="$1"
    local temp_file=$(mktemp)
    
    echo "  Updating $file..."
    
    # Create updated Cargo.toml with workspace dependencies
    awk '
    BEGIN { in_deps = 0 }
    
    /^\[dependencies\]/ { 
        in_deps = 1
        print $0
        next
    }
    
    /^\[/ && !/^\[dependencies\]/ {
        in_deps = 0
        print $0
        next
    }
    
    in_deps && /^wit-bindgen/ {
        print "wit-bindgen = { workspace = true }"
        next
    }
    
    in_deps && /^serde[[:space:]]*=/ {
        print "serde = { workspace = true }"
        next
    }
    
    in_deps && /^serde_json/ {
        print "serde_json = { workspace = true }"
        next
    }
    
    in_deps && /^component-metadata/ && !/build/ {
        print "component-metadata = { workspace = true }"
        next
    }
    
    in_deps && /^crossbeam-channel/ {
        print "crossbeam-channel = { workspace = true }"
        next
    }
    
    in_deps && /^lazy_static/ {
        print "lazy_static = { workspace = true }"
        next
    }
    
    in_deps && /^log[[:space:]]*=/ {
        print "log = { workspace = true }"
        next
    }
    
    in_deps && /^image[[:space:]]*=/ {
        print "image = { workspace = true }"
        next
    }
    
    in_deps && /^ndarray/ {
        print "ndarray = { workspace = true }"
        next
    }
    
    in_deps && /^bytemuck/ {
        print "bytemuck = { workspace = true }"
        next
    }
    
    # Keep all other lines as-is
    { print }
    ' "$file" > "$temp_file"
    
    mv "$temp_file" "$file"
    echo "    ✅ Updated $file"
}

# Update all component Cargo.toml files
find components/ -name "Cargo.toml" | while read file; do
    update_component_deps "$file"
done

echo "🎉 All component dependencies updated to use workspace versions!"