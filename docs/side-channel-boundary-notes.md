# Side-Channel Boundary Notes

This document defines the current side-channel boundary for SNARK_LAB.

## Current classification

SNARK_LAB is a production-grade research prototype.

It is not yet side-channel audited.

It must not be described as production-secure deployment software until side-channel review is completed.

## Public inputs

The following values are treated as public:

- protocol names
- statement sizes
- number of variables
- proof sizes
- commitment bytes
- verifier public inputs
- claimed opening values
- opening points
- SRS provenance metadata
- proof format versions
- transcript domain labels

## Potentially secret inputs

The following values may be secret depending on deployment context:

- witness vectors
- polynomial evaluation tables
- blinding scalars
- prover randomness
- unrevealed intermediate witness data
- private SRS trapdoor material, if any ceremony is not transparent

## Current implementation boundary

The repository currently prioritizes:

- algebraic correctness
- transcript binding
- parser hardening
- SRS validation
- malformed input rejection
- reproducible testing
- CI hardening

The repository does not currently claim:

- constant-time execution
- cache-timing resistance
- power-analysis resistance
- branch-predictor side-channel resistance
- memory-access side-channel resistance
- hardware side-channel resistance

## Rust unsafe boundary

Repository crates forbid unsafe code where added, and the production gate rejects unsafe Rust patterns in protocol crates and fuzz targets.

This reduces memory-unsafety risk.

It does not prove constant-time behavior.

## Field arithmetic boundary

Field and curve arithmetic are delegated to Arkworks.

Side-channel properties of Arkworks operations are outside this repository and must be reviewed separately before production deployment.

## Browser boundary

The browser visualizer is educational.

It uses small-field arithmetic and interactive UI state. It is not side-channel hardened and must not be used with production secrets.

## CLI boundary

CLI commands are for development, validation, reproducibility, and demo vectors.

Do not feed production secrets into CLI workflows until side-channel review is complete.

## Timing boundary

Benchmark timings are performance measurements.

They are not side-channel evidence.

A benchmark being stable or fast does not imply constant-time behavior.

## Required production review

Before production deployment, review:

- secret-dependent branches
- secret-dependent memory access
- variable-time field arithmetic
- scalar multiplication behavior
- proof generation with secret witness data
- blinding scalar generation
- RNG usage
- serialization failure timing
- parser rejection timing
- CLI handling of secret material
- operating-system and filesystem leakage
- logs and debug outputs

## Required evidence before production-secure claim

A production-secure claim requires:

- documented secret/public data classification
- side-channel review report
- constant-time assessment for secret-dependent paths
- dependency review for cryptographic arithmetic
- RNG review
- audit issue tracker
- remediation record
- release notes identifying side-channel status

## Current conclusion

SNARK_LAB has not completed side-channel review.

The correct current claim is:

    production-grade research prototype, not side-channel audited

The target future claim is:

    production-secure after external audit, side-channel review, and production SRS ceremony evidence
