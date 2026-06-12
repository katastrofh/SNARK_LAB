#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

TAG="${1:-}"
ACTION="${2:-}"

if [[ -z "$TAG" ]]; then
  echo "usage: scripts/prepare-release-tag.sh <tag> [--create]" >&2
  echo "example: scripts/prepare-release-tag.sh v0.2.0-rc.1" >&2
  exit 2
fi

if [[ ! "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]]; then
  echo "invalid tag format: $TAG" >&2
  echo "expected: vMAJOR.MINOR.PATCH or vMAJOR.MINOR.PATCH-rc.N" >&2
  exit 2
fi

if [[ -n "$(git status --short)" ]]; then
  echo "working tree is not clean" >&2
  git status --short >&2
  exit 1
fi

if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "tag already exists: $TAG" >&2
  exit 1
fi

scripts/check-release-checklist.sh
scripts/check-production-ready.sh

COMMIT="$(git rev-parse HEAD)"
LATEST_JSON="release-candidates/LATEST.json"

echo "release tag candidate"
echo "tag: $TAG"
echo "commit: $COMMIT"
echo "release evidence: $LATEST_JSON"

if [[ "$ACTION" == "--create" ]]; then
  git tag -a "$TAG" -m "SNARK_LAB $TAG release candidate

Commit: $COMMIT
Evidence: $LATEST_JSON

This tag does not claim production-secure status unless release notes explicitly include completed audit, side-channel review, and production SRS ceremony evidence."
  echo "created annotated tag: $TAG"
  echo "push with: git push origin $TAG"
else
  echo "dry run only"
  echo "to create tag, run: scripts/prepare-release-tag.sh $TAG --create"
fi
