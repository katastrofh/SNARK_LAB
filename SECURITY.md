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
