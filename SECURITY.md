# Security policy

## Scope

The Rust crates implement large-field protocol algebra and transcript ordering, while the browser is an educational F₉₇ visualizer. The repository is **not yet a complete production SNARK** because transparent oracle tables have not been replaced by a reviewed polynomial-commitment and opening backend.

Security-sensitive code includes:

- field encoding and challenge derivation in `crates/transcript`;
- statement/message ordering in `crates/sumcheck`, `crates/zerocheck`, and `crates/permcheck`;
- untrusted JSON limits and validation in `crates/interchange`;
- browser export and deployment security policy in `web/visualizer`.

## Supported versions

Only the latest commit on the default branch is supported while the project remains pre-1.0.

## Reporting a vulnerability

Do not open a public issue for an exploitable vulnerability. Use GitHub private vulnerability reporting for the repository. Include the affected commit, reproduction steps, impact, and any suggested mitigation. Avoid including private witness data or secrets in reports.

## Deployment requirements

- Serve the visualizer over HTTPS.
- Preserve the headers in `web/visualizer/public/_headers`, or configure equivalent headers at the hosting layer.
- Build with `npm ci` and `cargo build --locked --release`.
- Treat browser-exported F₉₇ JSON as untrusted educational input.
- Do not describe transparent-oracle proofs as succinct or zero knowledge.
- Pin and review any future commitment parameters and trusted setup artifacts.

## Cryptographic review gates

A production release is blocked until all of the following are complete:

1. a commitment-backed oracle abstraction and authenticated final openings;
2. a documented setup/parameter lifecycle for the selected commitment scheme;
3. canonical proof serialization with malformed-input and subgroup checks;
4. batch-verification and domain-separation review;
5. fuzzing and property testing of proof deserialization and verifier paths;
6. independent cryptographic audit and published test vectors.

## Threat model

See:

    docs/threat-model-and-security-notes.md

## Security review checklist

See:

    docs/security-review-checklist.md

The project is not audited. Do not use it for production funds, custody, mainnet systems, consensus-critical infrastructure, or security-critical deployments.

## Security proof sketch

See:

    docs/security-proof-sketch.md

The proof sketch documents assumptions and reductions for the research prototype. It is not an external audit.

## Release security boundary

Release and versioning policy:

    RELEASE.md
    VERSIONING.md
    CHANGELOG.md

All pre-1.0 releases are research-preview releases unless a later document explicitly says otherwise.

## Public test vectors

See:

    docs/public-test-vectors.md
    test-vectors/README.md

The committed vectors are deterministic regression artifacts, not production SRS material or audit evidence.

## Reference implementation comparison

See:

    docs/reference-implementation-comparison.md

Reference comparison tests are regression-hardening checks. They are not an external audit.

## Dependency update policy

See:

    docs/dependency-update-policy.md

Do not merge cryptographic dependency updates unless the entire dependency stack is upgraded coherently and all production gates pass.

## Fuzzing boundary

See:

    FUZZING.md
    docs/long-fuzz-campaign-notes.md

Fuzzing hardens malformed external byte parsers. It does not replace external audit or formal security review.

## Visualizer IPA boundary

See:

    docs/visualizer-real-ipa-flow.md

The browser IPA flow is educational and small-field. It is not an audit and not production deployment evidence.

## Visualizer demo boundary

See:

    docs/visualizer-polish-and-demo.md

Visualizer screenshots and GIFs are educational assets. They are not audit evidence.

## System visualizer boundary

See:

    docs/visualizer-system-flow.md

The System tab is an educational map of implemented components. It is not audit evidence or production deployment evidence.

## Side-channel and production deployment evidence

See:

    docs/side-channel-boundary-notes.md
    docs/production-deployment-evidence.md

Do not describe SNARK_LAB as production-secure until side-channel review, external audit, and production SRS ceremony evidence exist.

## Production SRS ceremony specification

See:

    docs/production-srs-ceremony-spec.md
    ceremony/README.md

The manifest verifier checks ceremony metadata and SRS digests. Production-security claims require real ceremony artifacts and external review.

## Deployment evidence pack

See:

    docs/deployment-evidence-pack.md
    deployment/README.md

Deployment evidence records what was actually run. It does not replace external audit, side-channel review, or production SRS ceremony evidence.

## Audit readiness packet

See:

    docs/audit-readiness-packet.md
    audits/packet/README.md

The audit packet prepares SNARK_LAB for external review. It does not itself mean an audit has been completed.

## Release-candidate evidence

See:

    docs/release-candidate-evidence-run.md
    release-candidates/README.md

Release-candidate evidence records executed checks. It does not replace external audit, side-channel review, or production SRS ceremony evidence.

## Release checklist and tagging

See:

    docs/production-release-checklist-and-tagging.md
    release/PRODUCTION_RELEASE_CHECKLIST.md

A release tag does not imply production-secure status unless audit, side-channel review, SRS ceremony evidence, and deployment evidence are complete.

## Release candidate security status

See:

    release/v0.2.0-rc.1.md

The release candidate is not production-secure. Production-secure status requires external audit, side-channel review, production SRS evidence, and deployment approval.

## Release artifacts and checksums

See:

    docs/github-release-artifacts-and-checksums.md

Checksums improve release reproducibility. They do not imply production-secure status without audit, side-channel review, and production SRS evidence.

## Long fuzz campaign evidence

See:

    docs/long-fuzz-campaign-evidence.md

Fuzz campaign evidence improves parser-hardening confidence. It does not replace external audit, side-channel review, or production SRS evidence.

## Production deployment guide

See:

    docs/production-deployment-guide.md
    docs/operator-runbook.md

The current release-candidate may be used for review and demonstration. It must not be deployed as production-secure software.
