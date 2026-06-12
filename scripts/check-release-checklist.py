#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    print(f"release checklist failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: str) -> Path:
    p = ROOT / path
    if not p.is_file():
        fail(f"missing file: {path}")
    return p


def require_executable(path: str) -> Path:
    p = require_file(path)
    if not p.stat().st_mode & 0o111:
        fail(f"not executable: {path}")
    return p


def parse_json(path: str) -> dict:
    p = require_file(path)
    try:
        return json.loads(p.read_text())
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path}: {exc}")


def main() -> None:
    required_files = [
        "release/README.md",
        "release/PRODUCTION_RELEASE_CHECKLIST.md",
        "release/RELEASE_NOTES_TEMPLATE.md",
        "release-candidates/README.md",
        "release-candidates/LATEST.md",
        "release-candidates/LATEST.json",
        "deployment/README.md",
        "audits/packet/README.md",
        "ceremony/production-srs-manifest.example.json",
        "docs/production-deployment-evidence.md",
        "docs/production-srs-ceremony-spec.md",
        "docs/release-candidate-evidence-run.md",
        "SECURITY.md",
        "CHANGELOG.md",
        "README.md",
        "Cargo.lock",
        "web/visualizer/package-lock.json",
    ]

    for path in required_files:
        require_file(path)

    required_scripts = [
        "scripts/check-production-ready.sh",
        "scripts/check-test-vectors.sh",
        "scripts/check-fuzz-targets.sh",
        "scripts/check-srs-ceremony-spec.sh",
        "scripts/check-deployment-evidence-pack.sh",
        "scripts/check-audit-readiness-packet.sh",
        "scripts/check-release-candidate-evidence.sh",
        "scripts/collect-deployment-evidence.sh",
        "scripts/summarize-latest-deployment-evidence.py",
    ]

    for path in required_scripts:
        require_executable(path)

    latest = parse_json("release-candidates/LATEST.json")

    expected_keys = [
        "schema",
        "status",
        "evidence_pack",
        "branch_under_evidence",
        "commit_under_evidence",
        "git_status_clean",
        "production_gate_passed",
        "public_vectors_passed",
        "srs_manifest_example_passed",
        "fuzz_targets_compile_passed",
        "external_audit_completed",
        "side_channel_review_completed",
        "production_srs_ceremony_completed",
        "production_secure",
    ]

    for key in expected_keys:
        if key not in latest:
            fail(f"release-candidates/LATEST.json missing key: {key}")

    checks = [
        "git_status_clean",
        "production_gate_passed",
        "public_vectors_passed",
        "srs_manifest_example_passed",
        "fuzz_targets_compile_passed",
    ]

    for key in checks:
        if latest[key] is not True:
            fail(f"release candidate evidence has {key}=false")

    if latest["production_secure"] is not False:
        fail("release candidate evidence must not claim production_secure=true")

    if latest["external_audit_completed"] is not False:
        fail("release candidate evidence must not claim external audit completion")

    if latest["side_channel_review_completed"] is not False:
        fail("release candidate evidence must not claim side-channel review completion")

    if latest["production_srs_ceremony_completed"] is not False:
        fail("release candidate evidence must not claim production SRS ceremony completion")

    commit = latest["commit_under_evidence"]
    if not isinstance(commit, str) or not re.fullmatch(r"[0-9a-f]{40}", commit):
        fail("commit_under_evidence must be a 40-character commit hash")

    print("release checklist is valid")


if __name__ == "__main__":
    main()
