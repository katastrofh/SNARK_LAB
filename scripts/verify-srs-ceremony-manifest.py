#!/usr/bin/env python3
import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

HEX64 = re.compile(r"^[0-9a-f]{64}$")

REQUIRED_TOP_LEVEL = [
    "manifest_version",
    "status",
    "scheme",
    "curve",
    "field",
    "max_variables",
    "generator_derivation",
    "artifacts",
    "participants",
    "transcript",
    "security_statement",
]

REQUIRED_GENERATOR = [
    "method",
    "domain_separator",
    "public_beacon",
]

REQUIRED_ARTIFACTS = [
    "final_srs_path",
    "final_srs_sha256",
    "verification_command",
]

REQUIRED_TRANSCRIPT = [
    "transcript_sha256",
    "transcript_path",
    "public_logs_path",
]

REQUIRED_SECURITY = [
    "trusted_setup_required",
    "toxic_waste_expected",
    "production_ready",
    "audit_required_before_production_claim",
]


def fail(message: str) -> None:
    print(f"manifest verification failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_keys(obj: dict, keys: list[str], where: str) -> None:
    for key in keys:
        if key not in obj:
            fail(f"missing {where}.{key}")


def require_hex64(value: str, where: str) -> None:
    if not isinstance(value, str) or not HEX64.match(value):
        fail(f"{where} must be a lowercase 64-character hex SHA-256 digest")


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description="Verify an SNARK_LAB SRS ceremony manifest.")
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--strict-production", action="store_true")
    args = parser.parse_args()

    try:
        manifest = json.loads(args.manifest.read_text())
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON: {exc}")

    require_keys(manifest, REQUIRED_TOP_LEVEL, "manifest")

    if manifest["scheme"] != "ipa-multilinear-pcs":
        fail("scheme must be ipa-multilinear-pcs")

    if manifest["curve"] != "BLS12-381":
        fail("curve must be BLS12-381")

    if manifest["field"] != "Fr":
        fail("field must be Fr")

    if not isinstance(manifest["max_variables"], int) or manifest["max_variables"] <= 0:
        fail("max_variables must be a positive integer")

    generator = manifest["generator_derivation"]
    artifacts = manifest["artifacts"]
    transcript = manifest["transcript"]
    security = manifest["security_statement"]

    require_keys(generator, REQUIRED_GENERATOR, "generator_derivation")
    require_keys(artifacts, REQUIRED_ARTIFACTS, "artifacts")
    require_keys(transcript, REQUIRED_TRANSCRIPT, "transcript")
    require_keys(security, REQUIRED_SECURITY, "security_statement")

    require_hex64(artifacts["final_srs_sha256"], "artifacts.final_srs_sha256")
    require_hex64(transcript["transcript_sha256"], "transcript.transcript_sha256")

    if not isinstance(manifest["participants"], list) or len(manifest["participants"]) == 0:
        fail("participants must be a non-empty array")

    for index, participant in enumerate(manifest["participants"]):
        if not isinstance(participant, dict):
            fail(f"participants[{index}] must be an object")
        for key in ["name", "role", "contribution_sha256"]:
            if key not in participant:
                fail(f"participants[{index}] missing {key}")
        require_hex64(participant["contribution_sha256"], f"participants[{index}].contribution_sha256")

    for key in REQUIRED_SECURITY:
        if not isinstance(security[key], bool):
            fail(f"security_statement.{key} must be boolean")

    if security["toxic_waste_expected"]:
        fail("IPA transparent parameter flow should not expect toxic waste")

    final_srs_path = artifacts["final_srs_path"]

    if final_srs_path:
        resolved = (args.manifest.parent / final_srs_path).resolve()
        if not resolved.exists():
            fail(f"final SRS file not found: {resolved}")
        digest = sha256_file(resolved)
        if digest != artifacts["final_srs_sha256"]:
            fail("final SRS SHA-256 digest does not match manifest")

    if args.strict_production:
        if manifest["status"] != "production":
            fail("strict production manifests must have status=production")
        if not final_srs_path:
            fail("strict production manifests must include final_srs_path")
        if artifacts["final_srs_sha256"] == "0" * 64:
            fail("strict production manifests must not use placeholder final_srs_sha256")
        if transcript["transcript_sha256"] == "0" * 64:
            fail("strict production manifests must not use placeholder transcript_sha256")
        if not security["production_ready"]:
            fail("strict production manifests must set production_ready=true")

    print(f"manifest ok: {args.manifest}")


if __name__ == "__main__":
    main()
