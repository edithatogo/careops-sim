#!/bin/sh
# Reuse the already-pinned Conductor submodule; do not install a mutable remote copy.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"
git submodule update --init -- extensions/conductor
expected=$(git ls-tree HEAD extensions/conductor | awk '{print $3}')
actual=$(git -C extensions/conductor rev-parse HEAD)
if [ -z "$expected" ] || [ "$expected" != "$actual" ]; then
  echo "Conductor checkout does not match the repository's reviewed Git pin" >&2
  exit 2
fi
if ! command -v gemini >/dev/null 2>&1; then
  echo "Gemini CLI is unavailable. Install it in your own environment first." >&2
  exit 3
fi
gemini extensions install "$root/extensions/conductor"
echo "Check installation with: gemini extensions list"
