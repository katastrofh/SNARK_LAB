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

## Production SRS placeholder policy

See:

    docs/production-srs-artifact-placeholder-policy.md
    srs/PRODUCTION_SRS_POLICY.md

Example manifests do not imply production SRS completion. Fake production SRS artifacts must not be committed.

## Fuzz campaign runner

See:

    docs/fuzz-nightly-runner-fix.md

Stable CI compiles fuzz targets. Nightly is required for actual sanitizer-backed cargo-fuzz campaign execution.

## Fuzz campaign runner hardening

See:

    docs/fuzz-campaign-runner-hardening.md

Failed fuzz runs are not production evidence. Crashes require triage and regression tests.

## IPA fuzz runtime requirements

See:

    docs/ipa-fuzz-target-runtime-fix.md

A successful smoke fuzz run is not production fuzz evidence. Long campaign logs still need to be archived and triaged.

## IPA proof decoder fuzz regression

See:

    docs/ipa-proof-decoder-fuzz-regression.md

Malformed IPA proof count fields must fail closed with decode errors, not panics.

## Nightly fuzz smoke evidence

See:

    docs/nightly-fuzz-smoke-evidence.md

Smoke fuzz evidence confirms the target can run briefly under nightly cargo-fuzz. It does not replace long fuzz campaigns or external audit.

## All fuzz targets smoke evidence

See:

    docs/all-fuzz-targets-smoke-evidence.md

All parser fuzz targets have short smoke evidence. Long-duration fuzz campaign evidence is still required for production-secure claims.

## Fuzz crash regression suite

See:

    docs/fuzz-crash-regression-suite.md

Known parser crashes must become regression tests and must return decode errors rather than panics.

## Public README security boundary

See:

    docs/readme-star-polish.md

The README is required to state that the release candidate is not production-secure and not externally audited.

## Visualizer screenshot assets

See:

    docs/visualizer-screenshot-assets.md

Visualizer screenshots are documentation assets only. They are not cryptographic evidence.

## Repository topic and badge polish

See:

    docs/repository-topic-and-badge-polish.md

Badges and discovery metadata must not claim audited production security, custody safety, or mainnet readiness.

## GitHub Release page finalization

See:

    docs/github-release-page-finalization.md

The release page must not claim external audit, production SRS completion, custody safety, mainnet readiness, or production-secure deployment.

## Manual GitHub Release publication evidence

See:

    docs/manual-github-release-publication-evidence.md

Release publication evidence confirms asset publication only. It does not claim production security.

## v0.2.0-rc.2 current-main release candidate

See:

    docs/release-candidate-rc2-current-main.md

v0.2.0-rc.2 is a review and reproducibility release candidate. It does not claim production security.

## GitHub Release rc2 publication evidence

See:

    docs/github-release-rc2-publication.md

Release publication evidence confirms asset publication only. It does not claim production security.

## Final project positioning and roadmap

See:

    docs/final-project-positioning-and-roadmap.md
    docs/project-positioning.md
    ROADMAP.md

The roadmap keeps future engineering and research work separate from production-security claims.

## Final repository health report

See:

    docs/final-repo-health-report.md
    docs/final-repo-health-report-notes.md

The health report summarizes repository evidence and remaining review blockers without making stronger security claims.

## Reviewer onboarding

See:

    REVIEWERS.md
    docs/reviewer-onboarding-guide.md
    docs/reviewer-onboarding-branch-notes.md

The onboarding guide helps reviewers inspect the repository without weakening the stated security boundary.

## Examples gallery

See:

    examples/README.md
    docs/examples-gallery.md

The examples are for review and education. They are not deployment instructions for security-critical settings.

## Paper-style technical overview

See:

    docs/paper-style-technical-overview.md
    docs/protocol-stack-summary.md
    docs/paper-style-technical-overview-notes.md

The overview explains the repository scope and limitations without making deployment claims.

## Post-freeze maintenance

See:

    FREEZE.md
    docs/post-freeze-maintenance-policy.md

The freeze is a project-management milestone. It does not change the security status of the repository.
