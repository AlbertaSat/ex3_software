#!/bin/bash

delete_dirs() {
  for dir in "$1"/*/; do
    # Check if it's a directory
    if [ -d "$dir" ]; then
      # Get the base directory name
      base_dir=$(basename "$dir")

      # Delete the directory if it's named 'logs' or ends with 'data' (This can be changed once we have better naming conventions)
      if [[ "$base_dir" == "logs" || "$base_dir" == *"data" ]]; then
        # Remove any trailing slash and delete the directory
        dir=$(realpath "$dir")
        echo "Deleting directory: $dir"
        rm -rf "$dir"
      else
        delete_dirs "$dir"
      fi
    fi
  done
}

# Start the process from the parent directory
delete_dirs "$(dirname "$(pwd)")"
