#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

RUN_GATE=1

for arg in "$@"; do
  case "$arg" in
    --skip-gate)
      RUN_GATE=0
      ;;
    *)
      echo "unknown argument: $arg" >&2
      echo "usage: scripts/collect-deployment-evidence.sh [--skip-gate]" >&2
      exit 2
      ;;
  esac
done

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="deployment/evidence/${STAMP}"

mkdir -p "$OUT"

run_capture() {
  local name="$1"
  shift

  echo "$*" > "${OUT}/${name}.cmd"
  "$@" > "${OUT}/${name}.log" 2>&1
}

{
  echo "# SNARK_LAB Deployment Evidence"
  echo
  echo "- generated_utc: ${STAMP}"
  echo "- branch: $(git branch --show-current)"
  echo "- commit: $(git rev-parse HEAD)"
  echo "- status: generated evidence pack"
  echo
  echo "This evidence pack records commands and outputs from the current checkout."
  echo "It does not imply production-security unless audit, SRS ceremony, side-channel review, and release checks are complete."
} > "${OUT}/SUMMARY.md"

git rev-parse HEAD > "${OUT}/commit.txt"
git branch --show-current > "${OUT}/branch.txt"
git status --short > "${OUT}/git-status-short.txt"
git status > "${OUT}/git-status.txt"

{
  echo "uname: $(uname -a)"
  echo "rustc: $(rustc --version 2>/dev/null || true)"
  echo "cargo: $(cargo --version 2>/dev/null || true)"
  echo "node: $(node --version 2>/dev/null || true)"
  echo "npm: $(npm --version 2>/dev/null || true)"
  echo "python3: $(python3 --version 2>/dev/null || true)"
} > "${OUT}/environment.txt"

run_capture cargo-tree cargo tree --locked
run_capture public-test-vectors scripts/check-test-vectors.sh
run_capture srs-ceremony-spec scripts/check-srs-ceremony-spec.sh
run_capture fuzz-targets scripts/check-fuzz-targets.sh

if [[ "$RUN_GATE" -eq 1 ]]; then
  run_capture production-readiness scripts/check-production-ready.sh
else
  echo "skipped by --skip-gate" > "${OUT}/production-readiness.log"
fi

{
  for path in \
    Cargo.lock \
    web/visualizer/package-lock.json \
    test-vectors/ipa-demo-v1.txt \
    ceremony/production-srs-manifest.example.json \
    README.md \
    SECURITY.md \
    CHANGELOG.md \
    RELEASE.md \
    VERSIONING.md
  do
    if [[ -f "$path" ]]; then
      sha256sum "$path"
    fi
  done
} > "${OUT}/tracked-artifact-sha256s.txt"

python3 - <<PY
import json
from pathlib import Path

out = Path("${OUT}")
attestation = {
    "schema": "snark-lab-release-evidence-v1",
    "status": "generated-not-production-attestation",
    "generated_utc": "${STAMP}",
    "branch": Path(out / "branch.txt").read_text().strip(),
    "commit": Path(out / "commit.txt").read_text().strip(),
    "git_status_clean": Path(out / "git-status-short.txt").read_text().strip() == "",
    "production_gate_log": "production-readiness.log",
    "public_vectors_log": "public-test-vectors.log",
    "srs_manifest_log": "srs-ceremony-spec.log",
    "fuzz_targets_log": "fuzz-targets.log",
    "artifact_sha256s": "tracked-artifact-sha256s.txt",
    "production_secure": False,
    "notes": "Generated evidence pack. Production-secure status requires external audit, side-channel review, production SRS evidence, and release approval."
}
Path(out / "attestation.json").write_text(json.dumps(attestation, indent=2) + "\\n")
PY

echo "wrote deployment evidence pack: ${OUT}"
