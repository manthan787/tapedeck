#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Usage: ./scripts/release.sh <version>  (e.g. 0.2.0)}"
TAG="v${VERSION}"
NAME="tapedeck"
ARCH="$(uname -m)"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ASSET="${NAME}-${TAG}-${OS}-${ARCH}.tar.gz"

# Ensure we're in the repo root
cd "$(git rev-parse --show-toplevel)"

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Error: uncommitted changes. Commit or stash first."
  exit 1
fi

# Check tag doesn't already exist
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "Error: tag $TAG already exists."
  exit 1
fi

# Update version in Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
cargo check --quiet 2>/dev/null || true

# Build release binary
echo "Building release binary..."
cargo build --release

# Package
echo "Packaging ${ASSET}..."
tar -czf "$ASSET" -C target/release "$NAME"

# Commit version bump, tag, and push
git add Cargo.toml Cargo.lock
git commit -m "Release ${TAG}"
git tag "$TAG"
git push
git push --tags

# Create GitHub release
echo "Creating GitHub release ${TAG}..."
gh release create "$TAG" "$ASSET" \
  --title "$TAG" \
  --generate-notes

# Cleanup
rm -f "$ASSET"

echo ""
echo "Released ${TAG}"
echo "https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/releases/tag/${TAG}"
