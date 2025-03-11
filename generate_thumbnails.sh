#!/bin/bash

set -x
set -e

IMGDIR="./data/uploads"
THMBDIR="./data/thumbnails"

# Check if convert (part of ImageMagick) is installed
if ! command -v convert &>/dev/null; then
  echo "Error: ImageMagick's 'convert' command could not be found. Please install ImageMagick."
  exit 1
fi

mkdir -p "$THMBDIR"

# Iterate over each file in the directory
for file in "$IMGDIR"/*; do
  # Check if it is a regular file (not a directory or symlink)
  if [[ -f "$file" ]]; then
    tf=$(basename "$file")
    # Use convert to resize image while preserving aspect ratio
    magick "$file" -resize "300x300" "$THMBDIR/${tf}_300.jpg" &
    magick "$file" -resize "300x300^" -gravity center -extent "300x300" "$THMBDIR/${tf}_s300.jpg" &
    magick "$file" -resize "100x100^" -gravity center -extent "100x100" "$THMBDIR/${tf}_s100.jpg" &
    wait
    # convert "$file" -resize "300x300" -gravity center -extent "${TARGET_SIZE}" "resized_$file"
  fi
done

echo "All images in the directory have been resized."
