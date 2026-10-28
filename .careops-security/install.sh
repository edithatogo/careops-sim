#!/bin/sh
set -eu
cd "$(git rev-parse --show-toplevel)"
existing=$(git config --get core.hooksPath || true)
if [ -n "$existing" ] && [ "$existing" != '.careops-security/hooks' ]; then
    echo 'Existing hook configuration preserved. Chain the privacy hooks explicitly after reviewing it.' >&2
    exit 1
fi
for hook in pre-commit pre-push; do
    if [ -f "$(git rev-parse --git-path "hooks/$hook")" ] && [ -z "$existing" ]; then
        echo 'Existing native hooks preserved. Integrate explicitly.' >&2
        exit 1
    fi
done
git config --local core.hooksPath .careops-security/hooks
printf 'Privacy hooks installed for this clone.\n'
