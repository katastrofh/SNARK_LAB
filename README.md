# SNARK_LAB

**Build, inspect, test, and benchmark SNARK protocol components.**

`SNARK_LAB` is a Rust + TypeScript research lab for understanding and engineering SNARK building blocks: Sumcheck, Zerocheck, PermCheck, streaming bottlenecks, transcript binding, and an experimental Inner Product Argument (IPA) polynomial-commitment path.

The Rust core is built with Arkworks primitives, BLS12-381 scalar-field support, Merlin Fiat–Shamir transcripts, checked serialization boundaries, local production gates, and a browser visualizer for interactive protocol inspection.

> **Security boundary:** this repository is serious protocol infrastructure and a production-grade research prototype, but it is **not audited** and must **not** be used to protect funds, production systems, or security-critical deployments. The code intentionally labels educational components, rejects fake success paths, and keeps unsupported cryptographic configurations explicit.

---

## Status

| Component                                          | Status        |
| -------------------------------------------------- | ------------- |
| Fiat–Shamir Sumcheck                               | Implemented   |
| Zerocheck reduction                                | Implemented   |
| PermCheck product/rational fingerprints            | Implemented   |
| Transparent multilinear oracle                     | Implemented   |
| Browser-to-Rust educational transcript interchange | Implemented   |
| IPA commitment equation                            | Implemented   |
| IPA reduction-round state                          | Implemented   |
| IPA vector and generator folding                   | Implemented   |
| IPA L/R curve commitments                          | Implemented   |
| IPA prover opening loop                            | Implemented   |
| IPA verifier reduction loop                        | Implemented   |
| IPA proof codec                                    | Implemented   |
| Blinded IPA opening path                           | Implemented   |
| Integrated IPA commit/open/verify API              | Implemented   |
| IPA CLI demo                                       | Implemented   |
| IPA SRS provenance validation                      | Implemented   |
| IPA SRS file loader                                | Implemented   |
| IPA SRS validation CLI                             | Implemented   |
| Negative malformed-proof fixtures                  | Implemented   |
| Randomized IPA roundtrip tests                     | Implemented   |
| CLI SRS tooling integration tests                  | Implemented   |
| Fuzzing                                            | Implemented   |
| Benchmark suite for IPA path                       | Implemented   |
| Security proof sketch / threat model               | Planned       |
| External audit                                     | Not performed |

---

## Quick Start

### Run the full production gate

```bash
scripts/check-production-ready.sh
```

This gate runs:

```text
cargo fmt
cargo clippy with warnings denied
cargo test
visualizer production build
unsafe Rust rejection
visualizer NaN-footgun rejection
git working-tree summary
```

### Run all Rust tests

```bash
cargo test --workspace
```

### Run the IPA commit/open/verify demo

```bash
cargo run -p snark-lab-cli -- ipa-demo
```

Expected output includes:

```text
ipa-demo: verified blinded IPA opening
variables=2
commitment_bytes=48
decoded_rounds=3
```

### Validate an IPA SRS file

```bash
cargo run -p snark-lab-cli -- ipa-srs-validate --curve bls12-381-g1 ./path/to/ipa.srs
```

This command loads an SRS file, decodes canonical curve points, checks provenance, verifies the canonical SHA-256 basis digest, and rejects malformed or non-production SRS material.

### Run the browser visualizer

```bash
cd web/visualizer
npm install
npm run dev
```

Open:

```text
http://localhost:5173
```

The browser lab uses small educational arithmetic where appropriate so protocol state remains inspectable. The Rust core uses Arkworks field and curve types.

---

## What Is Implemented?

| Protocol Component     | Rust Core                                                                                                                             | Browser Lab                               |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| Sumcheck               | Generic multilinear Sumcheck with transcript-bound round polynomials and Merlin challenges                                            | Step-by-step educational visualization    |
| Zerocheck              | Constraint table bound before equality-mixing challenge; delegates to Fiat–Shamir Sumcheck                                            | Toggle violated constraints               |
| PermCheck              | Transcript-bound β, γ, tagged product/rational fingerprints, explicit denominator-pole errors                                         | Compare product and rational fingerprints |
| Scribe-style pressure  | Large-field runtime benchmark plus explicit logical I/O model                                                                         | Product-tree vs. streaming traffic        |
| Transcript interchange | Educational JSON verifier isolated in `crates/interchange`                                                                            | Export current Sumcheck experiment        |
| IPA PCS                | Typed IPA commitment/opening path with checked curve commitments, proof codec, blinding extension, SRS provenance, and CLI validation | IPA PCS panel                             |

---

## Repository Layout

```text
snark-lab/
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
├── examples/transcripts/
├── notebooks/
├── docs/
├── scripts/
└── .github/workflows/
```

---

## CLI Commands

### Verify an educational browser transcript

```bash
cargo run -p snark-lab-cli -- verify-transcript examples/transcripts/sumcheck-valid.json
cargo run -p snark-lab-cli -- verify-transcript examples/transcripts/sumcheck-bad-round.json
```

The version-1 interchange format is deterministic for visualizer compatibility and is explicitly educational. It does not drive the Rust cryptographic protocol challenges.

### Run a real IPA backend demo

```bash
cargo run -p snark-lab-cli -- ipa-demo
```

This executes:

```text
commit
open
encode public opening
decode public opening
verify
```

### Validate an IPA SRS file

```bash
cargo run -p snark-lab-cli -- ipa-srs-validate [--curve bls12-381-g1] <path.srs>
```

Validation checks:

```text
file read
format magic
canonical SRS decode
curve-point decode
identity rejection
duplicate rejection
generator-count validation
provenance validation
canonical basis digest validation
```

Unsupported curves fail before file access.

---

## IPA PCS Path

The IPA path currently includes:

```text
typed curve-point wrapper
canonical compressed point serialization
curve generator basis validation
commitment equation
prover commitment path
evaluation-basis construction
opening statement binding
reduction-round state
L/R round commitment computation
vector folding
generator folding
prover opening loop
verifier reduction loop
final commitment relation check
proof shape validation
proof codec
blinded opening extension
integrated commit/open/verify API
SRS provenance validation
SRS file loader
SRS validation CLI
negative fixtures
randomized roundtrip tests
CLI tooling tests
```

The typed integrated IPA backend is the supported path. Older shape-only backend surfaces remain explicit and must not fake successful proving or verification without the required curve, SRS, and blinding material.

---

## SRS Provenance Model

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

The loader returns only `IpaVerifiedSrs`, meaning decoded material has already passed validation.

The repository does **not** claim that test fixture generators are production SRS material.

---

## Fiat–Shamir Ordering

Challenges are derived only after the relevant statement and prior prover message have been transcript-bound.

```text
bind protocol domain + public statement + oracle commitment
                              │
                              ▼
                     append prover message
                              │
                              ▼
                    derive challenge
                              │
                              ▼
                     append next message
                              │
                              ▼
                    derive next challenge
```

For Zerocheck, the constraint oracle is bound before the mixing point is derived.

For PermCheck, tagged columns are bound before β and γ are derived.

For IPA, opening statements and reduction-round commitments are absorbed before each folding challenge.

---

## Design Principles

1. **Messages before challenges.** Fiat–Shamir challenges are derived only after binding the relevant prior data.
2. **No fake success paths.** Unsupported cryptographic paths must return explicit errors.
3. **Checked curve material.** IPA curve points reject identity points and use canonical compressed serialization.
4. **SRS provenance is mandatory.** Production SRS material must be externally supplied or derived with auditable provenance.
5. **Educational components stay labeled.** Browser and interchange examples are not confused with the Rust cryptographic path.
6. **Fail closed on malformed inputs.** Decoders, proof checks, and CLI validators reject corrupt data.
7. **No unsafe Rust.** The production gate rejects unsafe Rust.
8. **Measured vs. modeled.** Runtime measurements and logical I/O models are kept distinct.

---

## Browser Visualizer

The browser workbench includes visual panels for:

```text
Sumcheck
Zerocheck
PermCheck
Scribe-style streaming pressure
IPA PCS
educational transcript export
```

Browser arithmetic is educational where small fields improve readability. The Rust protocol core is separate.

---

## Documentation

Key documentation lives in `docs/`, including:

```text
Fiat–Shamir Sumcheck
Zerocheck reduction
PermCheck fingerprints
Scribe streaming bottleneck
transparent oracle abstraction
IPA transcript rounds
IPA proof shape
IPA proof serialization
IPA generator basis
IPA curve types
IPA commitment equation
IPA prover commit path
IPA opening statement
IPA reduction rounds
IPA round commitments
IPA generator folding
IPA prover opening loop
IPA verifier opening loop
IPA blinding extension
IPA blinded prover/verifier path
IPA backend integration
IPA proof codec integration
IPA negative proof fixtures
IPA randomized roundtrip tests
IPA SRS provenance
IPA SRS loader
IPA SRS CLI
IPA SRS tooling tests
CI matrix and audit
local production gates
```

---

## Testing and Hardening

Current checks include:

```text
unit tests
negative malformed-proof tests
randomized roundtrip tests
CLI integration tests
canonical codec tests
production gate script
GitHub production-readiness workflow
Linux + macOS CI matrix
RustSec cargo-audit workflow
npm high-severity audit
Dependabot for Cargo, npm, and GitHub Actions
unsafe Rust rejection
visualizer production build
```

Planned hardening:

```text
fuzzing for proof and SRS decoders
larger IPA benchmark suite
memory profiling
dependency audit
CI matrix across Linux/macOS and stable Rust
threat model document
security theorem / proof sketch
side-channel review
external audit
```

---

## Benchmarks

Run the existing benchmark driver:

```bash
cargo run --release -p snark-lab-benches -- 18 8 3
```

The optional argument is `log₂(N)`, capped by the benchmark harness.

Current benchmark outputs distinguish measured runtime from modeled logical I/O. The benchmark binary includes PermCheck, Sumcheck, and IPA commit/open/verify timing. Future work will add memory profiles and hardware-counter-backed reports.

---

## Security Policy

See:

```text
SECURITY.md
```

This repository is not audited. Do not use it for production funds, mainnet systems, custody, consensus-critical infrastructure, or security-critical deployments.

---

## Roadmap

### Near-Term

```text
refresh README and public-facing docs
add SRS CLI tooling tests
add fuzzing targets for proof/SRS decoders
add IPA benchmark suite
add threat model and security proof sketch
update browser IPA panel to show real opening flow
```

### Research Direction

```text
compose Zerocheck + PermCheck + Sumcheck into a small HyperPlonk/Scribe-style proving pipeline
evaluate streaming and memory pressure in commitment-backed protocols
compare product-tree and rational PermCheck variants
measure prover I/O and memory bottlenecks
study Scribe-style polynomial commitment integration choices
```

---

## License

MIT

## Security boundary documents

See:

    docs/threat-model-and-security-notes.md
    docs/security-review-checklist.md

## Security proof sketch

See:

    docs/security-proof-sketch.md

This is a proof-outline for the implemented research prototype. It is not an audit and not a production deployment claim.

## Release and versioning

See:

    RELEASE.md
    VERSIONING.md
    CHANGELOG.md

Current releases are research-preview releases. They are not audited and are not production-secure deployment software.

## Public test vectors

See:

    test-vectors/README.md
    docs/public-test-vectors.md

The committed vectors are regression artifacts for the research prototype. They are not production SRS material.

## Reference implementation comparison

See:

    docs/reference-implementation-comparison.md

The reference tests compare selected production algebra against an independent slow implementation.

## Dependency update policy

See:

    docs/dependency-update-policy.md

Cryptographic dependencies are reviewed manually. Arkworks minor/major updates are intentionally not merged one crate at a time.
