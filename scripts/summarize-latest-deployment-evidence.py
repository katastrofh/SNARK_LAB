#!/usr/bin/env python3
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE_DIR = ROOT / "deployment" / "evidence"
OUT_DIR = ROOT / "release-candidates"


def fail(message: str) -> None:
    raise SystemExit(f"release-candidate summary failed: {message}")


def latest_pack() -> Path:
    if not EVIDENCE_DIR.exists():
        fail("deployment/evidence does not exist")

    packs = sorted(
        p for p in EVIDENCE_DIR.iterdir()
        if p.is_dir() and re.match(r"^\d{8}T\d{6}Z$", p.name)
    )

    if not packs:
        fail("no generated evidence packs found under deployment/evidence")

    return packs[-1]


def read_text(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(errors="replace")


def passed_log(pack: Path, name: str) -> bool:
    path = pack / f"{name}.log"
    if not path.exists():
        return False

    text = read_text(path).lower()
    if "skipped" in text:
        return False

    return True


def main() -> None:
    pack = latest_pack()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    commit = read_text(pack / "commit.txt").strip()
    branch = read_text(pack / "branch.txt").strip()
    git_status_short = read_text(pack / "git-status-short.txt").strip()
    environment = read_text(pack / "environment.txt").strip()

    checks = {
        "production_gate_passed": passed_log(pack, "production-readiness"),
        "public_vectors_passed": passed_log(pack, "public-test-vectors"),
        "srs_manifest_example_passed": passed_log(pack, "srs-ceremony-spec"),
        "fuzz_targets_compile_passed": passed_log(pack, "fuzz-targets"),
    }

    clean = git_status_short == ""
    status = "release-candidate-evidence-generated"

    summary_name = f"{pack.name}-summary.md"
    attestation_name = f"{pack.name}-attestation-summary.json"

    lines = [
        "# Release Candidate Evidence Summary",
        "",
        "## Identity",
        "",
        f"- Evidence pack: `{pack.relative_to(ROOT)}`",
        f"- Branch under evidence: `{branch}`",
        f"- Commit under evidence: `{commit}`",
        f"- Git status clean: `{str(clean).lower()}`",
        "",
        "## Check results",
        "",
        "| Check | Result |",
        "|---|---|",
        f"| Production readiness | `{str(checks['production_gate_passed']).lower()}` |",
        f"| Public test vectors | `{str(checks['public_vectors_passed']).lower()}` |",
        f"| SRS ceremony manifest example | `{str(checks['srs_manifest_example_passed']).lower()}` |",
        f"| Fuzz target compile | `{str(checks['fuzz_targets_compile_passed']).lower()}` |",
        "",
        "## Environment",
        "",
        "    " + environment.replace("\n", "\n    "),
        "",
        "## Artifact digests",
        "",
        "See raw generated file:",
        "",
        f"    {pack.relative_to(ROOT)}/tracked-artifact-sha256s.txt",
        "",
        "## Conclusion",
        "",
        f"Status: `{status}`",
        "",
        "Production secure: `false`",
        "",
        "This summary records a release-candidate evidence run.",
        "",
        "It does not claim external audit completion, side-channel review completion, or production SRS ceremony completion.",
        "",
    ]

    summary = "\n".join(lines)

    attestation = {
        "schema": "snark-lab-release-candidate-summary-v1",
        "status": status,
        "evidence_pack": str(pack.relative_to(ROOT)),
        "branch_under_evidence": branch,
        "commit_under_evidence": commit,
        "git_status_clean": clean,
        "production_gate_passed": checks["production_gate_passed"],
        "public_vectors_passed": checks["public_vectors_passed"],
        "srs_manifest_example_passed": checks["srs_manifest_example_passed"],
        "fuzz_targets_compile_passed": checks["fuzz_targets_compile_passed"],
        "external_audit_completed": False,
        "side_channel_review_completed": False,
        "production_srs_ceremony_completed": False,
        "production_secure": False,
    }

    (OUT_DIR / summary_name).write_text(summary)
    (OUT_DIR / attestation_name).write_text(json.dumps(attestation, indent=2) + "\n")
    (OUT_DIR / "LATEST.md").write_text(summary)
    (OUT_DIR / "LATEST.json").write_text(json.dumps(attestation, indent=2) + "\n")

    print(f"wrote {OUT_DIR / summary_name}")
    print(f"wrote {OUT_DIR / attestation_name}")
    print(f"updated {OUT_DIR / 'LATEST.md'}")
    print(f"updated {OUT_DIR / 'LATEST.json'}")


if __name__ == "__main__":
    main()
