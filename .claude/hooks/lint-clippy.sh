#!/usr/bin/env bash
# PostToolUse hook: runs cargo clippy + cargo fmt --check on edited Rust files.
# Reads the Claude tool-use JSON payload from stdin.

set -euo pipefail

# Parse the file path from the JSON payload on stdin
INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Skip if no file path found
if [[ -z "$FILE" ]]; then
  exit 0
fi

# Only process Rust files
if [[ "$FILE" != *.rs ]]; then
  exit 0
fi

echo "Lint/fmt check: $FILE"

# Format check
echo "-> cargo fmt --check"
cargo fmt --check 2>&1 || true

# Clippy lint
echo "-> cargo clippy"
cargo clippy -- -D warnings 2>&1 || true

echo "Done"
