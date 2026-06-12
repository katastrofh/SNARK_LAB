<!-- SNARK_LAB_README_PUBLIC_V2 -->

# SNARK_LAB

<p align="center">
  <img alt="status: research prototype" src="https://img.shields.io/badge/status-research%20prototype-blue">
  <img alt="release: v0.2.0-rc.2" src="https://img.shields.io/badge/release-v0.2.0--rc.2-orange">
  <img alt="language: Rust" src="https://img.shields.io/badge/language-Rust-informational">
  <img alt="fuzzing: smoke and regressions" src="https://img.shields.io/badge/fuzzing-smoke%20%2B%20regressions-success">
  <img alt="visualizer: available" src="https://img.shields.io/badge/visualizer-available-success">
  <img alt="license: MIT" src="https://img.shields.io/badge/license-MIT-lightgrey">
</p>

<p align="center">
  <strong>Research prototype.</strong>
  Not audited production-secure software.
</p>

**SNARK_LAB is a Rust protocol lab for SNARK building blocks: Sumcheck, Zerocheck, PermCheck, and IPA polynomial commitments.**

It connects the math of interactive proofs to executable Rust code, transcript-bound proof flows, public test vectors, fuzzing evidence, release-candidate artifacts, and an educational browser visualizer.

The goal is simple: make SNARK protocol mechanics inspectable.

---

## Why this repository matters

Most SNARK learning material stops at equations. Most production libraries hide the protocol mechanics behind APIs.

SNARK_LAB sits in the middle:

* readable Rust implementations of core SNARK components
* Fiat–Shamir transcript ordering
* Sumcheck, Zerocheck, and PermCheck flows
* an IPA polynomial-commitment research path
* malformed-proof rejection tests
* canonical proof/SRS decoding boundaries
* fuzz targets and fuzz regression tracking
* public vectors and reference comparisons
* release-candidate evidence
* a browser visualizer for protocol flow

This repository is built for students, researchers, reviewers, and engineers who want to understand how SNARK components fit together.

---

## Current status

| Area                         | Status                                      |
| ---------------------------- | ------------------------------------------- |
| Sumcheck                     | Implemented and tested                      |
| Zerocheck                    | Implemented and tested                      |
| PermCheck                    | Implemented and tested                      |
| IPA PCS path                 | Implemented as a research prototype         |
| IPA proof codecs             | Fuzzed and regression-tested                |
| SRS loader/provenance        | Implemented with production-boundary checks |
| Browser visualizer           | Implemented                                 |
| Public test vectors          | Implemented                                 |
| Release evidence             | Implemented                                 |
| Current release candidate    | `v0.2.0-rc.2`                               |
| External audit               | Not completed                               |
| Production-secure deployment | Not claimed                                 |

---

## What this is

SNARK_LAB is:

* a reviewer-facing research prototype
* a protocol-learning laboratory
* a reproducible SNARK component testbed
* an evidence-driven release-candidate artifact
* a bridge between SNARK math and executable protocol code

## What this is not

SNARK_LAB is not:

* audited deployment-ready cryptographic software
* mainnet-ready cryptographic infrastructure
* custody-safe software
* a replacement for external review
* a production SRS ceremony
* a library to protect real funds or consensus-critical systems

Do **not** use this repository for production funds, custody, consensus-critical systems, or security-critical deployment.

---

## Quickstart

Clone the repository:

```bash
git clone https://github.com/katastrofh/SNARK_LAB.git
cd SNARK_LAB
```

Run the full repository gate:

```bash
scripts/check-production-ready.sh
```

Run all Rust tests:

```bash
cargo test --workspace
```

Run the IPA commit/open/verify demo:

```bash
cargo run -p snark-lab-cli -- ipa-demo
```

Run the visualizer:

```bash
cd web/visualizer
npm ci
npm run dev
```

Open:

```text
http://localhost:5173
```

Build release artifacts for the current release candidate:

```bash
scripts/build-github-release-artifacts.sh v0.2.0-rc.2
```

---

## Protocol map

| Protocol/component | Purpose                                                       |
| ------------------ | ------------------------------------------------------------- |
| Sumcheck           | Proves claims about sums over Boolean hypercubes              |
| Zerocheck          | Reduces constraint satisfaction to polynomial zero checks     |
| PermCheck          | Checks multiset/permutation consistency                       |
| Transparent oracle | Provides simple inspectable opening/verification flow         |
| IPA PCS            | Commits to multilinear polynomials and proves openings        |
| SRS tooling        | Validates public-parameter provenance and artifact boundaries |
| Interchange format | Connects the educational visualizer to deterministic examples |

---

## Visualizer screenshots

The visualizer shows how the protocol components fit together.

<p align="center">
  <img src="docs/assets/visualizer/system-flow.png" alt="SNARK_LAB system flow visualizer" width="45%">
  <img src="docs/assets/visualizer/ipa-flow.png" alt="SNARK_LAB IPA flow visualizer" width="45%">
</p>

<p align="center">
  <img src="docs/assets/visualizer/sumcheck-flow.png" alt="SNARK_LAB sumcheck flow visualizer" width="70%">
</p>

---

## Evidence and hardening

| Evidence layer             | Location                                      |
| -------------------------- | --------------------------------------------- |
| Production-readiness gate  | `scripts/check-production-ready.sh`           |
| GitHub workflows           | `.github/workflows/`                          |
| Release candidate evidence | `release-candidates/LATEST.md`                |
| Current release notes      | `release/v0.2.0-rc.2.md`                      |
| GitHub release page draft  | `release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md`  |
| Public vectors             | `test-vectors/`                               |
| Reference comparisons      | `docs/reference-implementation-comparison.md` |
| Fuzz targets               | `fuzz/fuzz_targets/`                          |
| Fuzz smoke evidence        | `fuzz/smoke-evidence/`                        |
| Fuzz crash regressions     | `fuzz/regressions/`                           |
| SRS policy                 | `srs/PRODUCTION_SRS_POLICY.md`                |
| Deployment guide           | `docs/production-deployment-guide.md`         |
| Audit packet               | `audits/packet/README.md`                     |

---

## Repository layout

```text
SNARK_LAB/
├── crates/
│   ├── field/          # default BLS12-381 scalar field helpers
│   ├── multilinear/    # dense multilinear extensions and equality basis
│   ├── transcript/     # Merlin Fiat–Shamir abstraction
│   ├── sumcheck/       # generic transcript-bound Sumcheck
│   ├── zerocheck/      # transcript-ordered equality reduction
│   ├── permcheck/      # product/rational permutation fingerprints
│   ├── oracle/         # transparent oracle + IPA PCS infrastructure
│   ├── interchange/    # educational browser JSON verifier
│   ├── cli/            # transcript verifier, IPA demo, SRS validator
│   └── benches/        # runtime and logical I/O benchmarks
├── web/visualizer/     # React + TypeScript protocol workbench
├── examples/           # guided examples and transcripts
├── test-vectors/       # deterministic public regression vectors
├── fuzz/               # fuzz targets, smoke evidence, regressions
├── docs/               # protocol notes and evidence documentation
├── release/            # release notes and GitHub release pages
├── release-candidates/ # release-candidate evidence
├── scripts/            # local gates and artifact tooling
└── .github/workflows/  # CI, audit, and release workflows
```

---

## Implemented protocol areas

### Sumcheck

* Fiat–Shamir transcript-bound round challenges
* multilinear table support
* deterministic proof serialization
* malformed-round rejection tests
* transparent-oracle integration path

### Zerocheck

* constraint table binding before mixing challenge
* reduction to Sumcheck
* rejected nonzero constraint tables
* educational visualizer flow

### PermCheck

* product/rational fingerprints
* transcript-bound β and γ challenges
* explicit denominator-pole errors
* permutation/mutation rejection tests

### IPA polynomial-commitment path

The IPA path currently includes:

* typed curve-point wrapper
* canonical compressed point serialization
* generator-basis validation
* commitment equation
* prover commitment path
* evaluation-basis construction
* opening-statement binding
* reduction-round state
* L/R round commitment computation
* vector folding
* generator folding
* prover opening loop
* verifier reduction loop
* final commitment relation check
* proof shape validation
* proof codec
* blinded opening extension
* integrated commit/open/verify API
* SRS provenance validation
* SRS file loader
* SRS validation CLI
* negative proof fixtures
* randomized roundtrip tests
* fuzz regression tests

The typed integrated IPA backend is the supported research path. Unsupported cryptographic configurations fail explicitly rather than pretending to verify.

---

## SRS provenance boundary

Production SRS material must have provenance.

Accepted source types:

```text
ExternalTrustedSetup
HashToCurveDerivation
```

Rejected source type:

```text
KnownDiscreteLogTestFixture
```

The canonical SRS digest binds:

```text
digest domain version
variable count
polynomial generators
evaluation generators
blinding generator
```

The loader returns only verified SRS material after validation.

The repository does **not** claim that test fixture generators are production SRS material.

---

## Design principles

1. **Messages before challenges.** Fiat–Shamir challenges are derived only after binding the relevant statement and prior prover message.
2. **No fake success paths.** Unsupported cryptographic paths return explicit errors.
3. **Checked curve material.** IPA curve points reject identity points and use canonical compressed serialization.
4. **SRS provenance is mandatory.** Production SRS material must be externally supplied or derived with auditable provenance.
5. **Educational components stay labeled.** Browser and interchange examples are not confused with the Rust cryptographic path.
6. **Malformed inputs fail closed.** Decoders, proof checks, and CLI validators reject corrupt data.
7. **No unsafe Rust.** The production-readiness gate rejects unsafe Rust.
8. **Measured and modeled results stay separate.** Runtime measurements and logical I/O models are not conflated.

---

## Testing and CI

Current checks include:

* unit tests
* negative malformed-proof tests
* randomized IPA roundtrip tests
* CLI integration tests
* canonical codec tests
* public test-vector checks
* independent reference comparisons
* fuzz target compile checks
* fuzz smoke evidence checks
* fuzz crash regression tests
* visualizer production build
* GitHub production-readiness workflow
* Linux/macOS CI matrix
* RustSec cargo-audit workflow
* npm audit for the visualizer
* unsafe Rust rejection
* visualizer `Number.isNaN` footgun rejection

Run the local gate:

```bash
scripts/check-production-ready.sh
```

---

## Suggested reading order

For a quick review path:

1. `README.md`
2. `REVIEWERS.md`
3. `docs/project-positioning.md`
4. `docs/final-repo-health-report.md`
5. `docs/production-readiness-index.md`
6. `docs/security-review-checklist.md`
7. `docs/paper-style-technical-overview.md`
8. `release/v0.2.0-rc.2.md`
9. `web/visualizer/`

For protocol internals:

1. `crates/sumcheck/`
2. `crates/zerocheck/`
3. `crates/permcheck/`
4. `crates/oracle/`
5. `crates/cli/`
6. `test-vectors/`
7. `fuzz/`

---

## Benchmarks

Run the benchmark driver:

```bash
cargo run --release -p snark-lab-benches -- 18 8 3
```

The benchmark binary includes PermCheck, Sumcheck, and IPA commit/open/verify timing. Benchmark outputs distinguish measured runtime from modeled logical I/O.

---

## Release candidate

Current release candidate:

```text
v0.2.0-rc.2
```

See:

```text
release/v0.2.0-rc.2.md
release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md
release/publication/v0.2.0-rc.2/README.md
```

Release candidates are for protocol review, artifact review, reproducibility checks, and audit preparation. They are not production-secure deployment releases.

---

## Security policy

See:

```text
SECURITY.md
docs/threat-model-and-security-notes.md
docs/security-review-checklist.md
docs/security-proof-sketch.md
```

This repository is not audited. Do not use it for production funds, mainnet systems, custody, consensus-critical infrastructure, or security-critical deployments.

---

## Roadmap

See:

```text
ROADMAP.md
docs/project-positioning.md
docs/final-repo-health-report.md
```

Near-term work:

* external review
* longer fuzz campaigns
* benchmark reports
* memory profiling
* side-channel review notes
* protocol-composition experiments
* clearer HyperPlonk/Scribe-style pipeline integration

---

## License

MIT
