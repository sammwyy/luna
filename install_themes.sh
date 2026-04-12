#!/bin/bash

# Install luna themes

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Target directory for themes
TARGET_DIR="$HOME/.luna/themes"

# Create target directory if it doesn't exist
mkdir -p "$TARGET_DIR"

# Copy all .lua files from the source themes directory to the target directory
cp "$SCRIPT_DIR"/assets/themes/*.lua "$TARGET_DIR"/

echo "Installed themes to $TARGET_DIR"