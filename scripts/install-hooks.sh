#!/bin/sh
# Configure git to use the shared hooks in .githooks/
# Usage: ./scripts/install-hooks.sh

set -e

git config core.hooksPath .githooks
echo "Git hooks configured (core.hooksPath → .githooks)"
