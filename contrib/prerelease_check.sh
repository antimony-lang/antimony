#!/bin/bash

# Get the current branch name
current_branch=$(git branch --show-current)

echo "Checking current branch: $current_branch"
if [ "$current_branch" != "master" ]; then
    echo "Error: Not on master branch. Currently on '$current_branch'."
    exit 1
fi

# Run cargo clippy
echo "Running cargo clippy..."
if ! cargo clippy; then
    echo "Error: cargo clippy failed."
    exit 1
fi

# Run cargo fmt
echo "Running cargo fmt..."
if ! cargo fmt -- --check; then
    echo "Error: cargo fmt failed."
    exit 1
fi

# Run cargo test
echo "Running cargo test..."
if ! cargo test; then
    echo "Error: cargo test failed."
    exit 1
fi

echo "All checks passed successfully."