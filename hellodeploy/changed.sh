#!/bin/bash

# Get the current commit hash
NEW_COMMIT=$(git rev-parse HEAD)

# Get the parent commit hash of the current commit
OLD_COMMIT=$(git rev-parse HEAD^)

# Define the path you want to check
TARGET_PATH=$1

# Check if the path has changed between commits
if git diff --name-only $OLD_COMMIT $NEW_COMMIT -- $TARGET_PATH | grep -q $TARGET_PATH; then
    echo "true"
else
    echo "false"
fi