#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

TAG="${1:-}"

if [[ -z "$TAG" ]]; then
  echo "usage: scripts/build-github-release-artifacts.sh <tag>" >&2
  echo "example: scripts/build-github-release-artifacts.sh v0.2.0-rc.1" >&2
  exit 2
fi

if ! git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "tag not found: $TAG" >&2
  exit 1
fi

OUT="dist/releases/$TAG"
rm -rf "$OUT"
mkdir -p "$OUT"

COMMIT="$(git rev-list -n 1 "$TAG")"

echo "building release artifacts for $TAG"
echo "commit: $COMMIT"
echo "output: $OUT"

git archive --format=tar.gz --prefix="SNARK_LAB-$TAG/" -o "$OUT/SNARK_LAB-$TAG.source.tar.gz" "$TAG"
git archive --format=zip --prefix="SNARK_LAB-$TAG/" -o "$OUT/SNARK_LAB-$TAG.source.zip" "$TAG"

{
  echo "SNARK_LAB release artifact manifest"
  echo
  echo "tag: $TAG"
  echo "commit: $COMMIT"
  echo "generated_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo
  echo "This artifact bundle is for review and release-candidate validation."
  echo "It does not claim production-secure status."
} > "$OUT/MANIFEST.txt"

git show --no-patch --pretty=fuller "$TAG" > "$OUT/TAG_INFO.txt"

if git cat-file -e "$TAG:release/$TAG.md" 2>/dev/null; then
  git show "$TAG:release/$TAG.md" > "$OUT/RELEASE_NOTES.md"
else
  echo "release notes not found at release/$TAG.md" > "$OUT/RELEASE_NOTES.md"
fi

if git cat-file -e "$TAG:release-candidates/LATEST.md" 2>/dev/null; then
  git show "$TAG:release-candidates/LATEST.md" > "$OUT/RELEASE_CANDIDATE_EVIDENCE.md"
fi

if git cat-file -e "$TAG:release-candidates/LATEST.json" 2>/dev/null; then
  git show "$TAG:release-candidates/LATEST.json" > "$OUT/RELEASE_CANDIDATE_EVIDENCE.json"
fi

if git cat-file -e "$TAG:ceremony/production-srs-manifest.example.json" 2>/dev/null; then
  git show "$TAG:ceremony/production-srs-manifest.example.json" > "$OUT/SRS_MANIFEST_EXAMPLE.json"
fi

if git cat-file -e "$TAG:SECURITY.md" 2>/dev/null; then
  git show "$TAG:SECURITY.md" > "$OUT/SECURITY.md"
fi

if git cat-file -e "$TAG:CHANGELOG.md" 2>/dev/null; then
  git show "$TAG:CHANGELOG.md" > "$OUT/CHANGELOG.md"
fi

(
  cd "$OUT"
  find . -maxdepth 1 -type f ! -name SHA256SUMS -printf '%P\0' | sort -z | xargs -0 sha256sum > SHA256SUMS
)

echo "release artifacts written to $OUT"
echo
cat "$OUT/SHA256SUMS"
