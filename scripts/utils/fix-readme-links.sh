#!/bin/bash

# Fix navigation links in README level files
echo "Fixing navigation links in README level files..."

# Get the directory where the script is located
SCRIPT_DIR="$(dirname "$0")"
PROJECT_ROOT="$SCRIPT_DIR/../.."
DOCS_DIR="$PROJECT_ROOT/docs/readme-levels"

# Fix links in all README.L*.md files
for i in {0..9}; do
    FILE="$DOCS_DIR/README.L${i}.md"
    if [ -f "$FILE" ]; then
        echo "Fixing links in README.L${i}.md..."
        
        # Fix links to other level READMEs (they reference each other as ./README.LX.md)
        for j in {0..9}; do
            sed -i '' "s|\./README\.L${j}\.md|./README.L${j}.md|g" "$FILE"
        done
        
        # Fix any links back to parent directories that might be broken
        sed -i '' "s|\.\./\.\./README\.md|../../README.md|g" "$FILE"
        sed -i '' "s|\.\./README\.md|../../README.md|g" "$FILE"
    fi
done

echo "Navigation links fixed!"