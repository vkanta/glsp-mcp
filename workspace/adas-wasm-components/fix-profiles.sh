#!/bin/bash
# Fix duplicate profile sections in component Cargo.toml files

set -e

echo "🔧 Fixing duplicate profile sections in component Cargo.toml files..."

# Find all Cargo.toml files with [profile.release] sections
FILES=$(find components/ -name "Cargo.toml" -exec grep -l "\[profile\.release\]" {} \;)

for file in $FILES; do
    echo "  Fixing $file..."
    
    # Create a temporary file
    temp_file=$(mktemp)
    
    # Remove everything from [profile.release] to the end of file
    # and replace with a comment about workspace inheritance
    awk '
    /^\[profile\.release\]/ { 
        print "# Profile configuration inherited from workspace"
        exit 
    }
    { print }
    ' "$file" > "$temp_file"
    
    # Replace the original file
    mv "$temp_file" "$file"
    
    echo "    ✅ Fixed $file"
done

echo "🎉 All profile duplications fixed!"
echo "📋 Components now inherit profile settings from workspace Cargo.toml"