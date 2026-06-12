#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-v0.2.0-rc.1}"
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

ASSET_DIR="dist/releases/${TAG}"
BODY="release/GITHUB_RELEASE_PAGE_${TAG}.md"

if [[ ! -f "$BODY" ]]; then
  echo "missing release body: $BODY" >&2
  exit 1
fi

echo "# 1. Build release assets:"
echo "scripts/build-github-release-artifacts.sh ${TAG}"
echo
echo "# 2. Verify asset checksums:"
echo "cd ${ASSET_DIR}"
echo "sha256sum -c SHA256SUMS"
echo "cd -"
echo
echo "# 3. Create GitHub release:"
echo "gh release create ${TAG} \\"
echo "  --repo katastrofh/SNARK_LAB \\"
echo "  --title \"SNARK_LAB ${TAG}\" \\"
echo "  --notes-file ${BODY} \\"
echo "  ${ASSET_DIR}/SNARK_LAB-${TAG}.source.tar.gz \\"
echo "  ${ASSET_DIR}/SNARK_LAB-${TAG}.source.zip \\"
echo "  ${ASSET_DIR}/SHA256SUMS \\"
echo "  ${ASSET_DIR}/MANIFEST.txt \\"
echo "  ${ASSET_DIR}/TAG_INFO.txt \\"
echo "  ${ASSET_DIR}/RELEASE_NOTES.md \\"
echo "  ${ASSET_DIR}/RELEASE_CANDIDATE_EVIDENCE.md \\"
echo "  ${ASSET_DIR}/RELEASE_CANDIDATE_EVIDENCE.json \\"
echo "  ${ASSET_DIR}/SRS_MANIFEST_EXAMPLE.json \\"
echo "  ${ASSET_DIR}/CHANGELOG.md \\"
echo "  ${ASSET_DIR}/SECURITY.md"
