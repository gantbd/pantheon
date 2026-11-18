#!/bin/sh
set -eu

# Repository trees are source distributions. Runtime state and model payloads
# belong in explicitly configured private directories outside the checkout.
limit_bytes=10485760

git ls-files | while IFS= read -r tracked_file; do
  case "$tracked_file" in
    target/*|*/target/*|.dev/*|*/.dev/*|.hestia-artifacts/*|*/.hestia-artifacts/*|*.gguf|*.safetensors|*.sock|*.log|.env|.env.*|*/.env|*/.env.*)
      echo "repository hygiene refused tracked runtime/generated path: $tracked_file" >&2
      exit 1
      ;;
  esac
  size=$(wc -c <"$tracked_file")
  if [ "$size" -gt "$limit_bytes" ]; then
    echo "repository hygiene refused tracked file over 10 MiB: $tracked_file" >&2
    exit 1
  fi
done
