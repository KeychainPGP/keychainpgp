#!/usr/bin/env bash
#
# bump-version.sh — Bump the KeychainPGP version across all manifests,
#                    commit, tag, push, and create a GitHub release.
#
# Usage:  ./scripts/bump-version.sh <new-version>
# Example: ./scripts/bump-version.sh 0.2.0
#
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <new-version>"
  echo "Example: $0 0.2.0"
  exit 1
fi

NEW_VERSION="$1"

# Validate semver-ish format
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
  echo "Error: version must be in semver format (e.g. 0.2.0 or 1.0.0-beta.1)"
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> Bumping version to $NEW_VERSION"

# 1. Cargo.toml (workspace)
sed -i.bak -E 's/^(version = ")[^"]+(")/\1'"$NEW_VERSION"'\2/' Cargo.toml && rm -f Cargo.toml.bak
echo "  Updated Cargo.toml"

# 2. tauri.conf.json
sed -i.bak -E 's/("version": ")[^"]+(")/\1'"$NEW_VERSION"'\2/' crates/keychainpgp-ui/tauri.conf.json && rm -f crates/keychainpgp-ui/tauri.conf.json.bak
echo "  Updated tauri.conf.json"

# 3. Frontend package.json
sed -i.bak -E 's/("version": ")[^"]+(")/\1'"$NEW_VERSION"'\2/' crates/keychainpgp-ui/frontend/package.json && rm -f crates/keychainpgp-ui/frontend/package.json.bak
echo "  Updated frontend package.json"

# 4. Web package.json
sed -i.bak -E 's/("version": ")[^"]+(")/\1'"$NEW_VERSION"'\2/' web/package.json && rm -f web/package.json.bak
echo "  Updated web package.json"

# 5. Update Cargo.lock
cargo generate-lockfile --quiet 2>/dev/null || true
echo "  Updated Cargo.lock"

echo ""
echo "==> Committing and tagging"
git add Cargo.toml Cargo.lock \
       crates/keychainpgp-ui/tauri.conf.json \
       crates/keychainpgp-ui/frontend/package.json \
       web/package.json
git commit -m "chore: bump version to v$NEW_VERSION"
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

echo ""
echo "==> Pushing to remote"
git push && git push --tags

echo ""
echo "==> Creating GitHub release"
gh release create "v$NEW_VERSION" \
  --title "v$NEW_VERSION" \
  --generate-notes

echo ""
echo "Done! Version bumped to v$NEW_VERSION and release created."
