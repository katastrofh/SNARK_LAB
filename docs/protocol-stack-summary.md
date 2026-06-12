# Protocol Stack Summary

This document gives a compact map of the SNARK_LAB protocol stack.

## Layer 1: Field and multilinear utilities

These provide the arithmetic and evaluation primitives needed by the higher-level protocols.

Review focus:

- field element handling
- multilinear table shape
- equality polynomial behavior
- evaluation correctness

## Layer 2: Sumcheck

Sumcheck reduces a claimed hypercube sum to a sequence of univariate checks.

Review focus:

- claimed sum binding
- round message validation
- challenge derivation
- final evaluation check
- tampering rejection

## Layer 3: Zerocheck

Zerocheck validates that a constraint table vanishes over the Boolean hypercube.

Review focus:

- zero versus nonzero table behavior
- oracle binding
- challenge timing
- rejection behavior

## Layer 4: PermCheck

PermCheck validates tagged permutation relations.

Review focus:

- tagged value construction
- permutation consistency
- denominator pole handling
- mutation rejection

## Layer 5: IPA polynomial commitments

The IPA path connects polynomial commitments to opening proofs.

Review focus:

- commitment equation
- opening statement binding
- generator folding
- challenge transcript
- verifier rejection paths

## Layer 6: Serialization and fuzzing

Encoded proofs and SRS files are handled through explicit codecs.

Review focus:

- canonical roundtrips
- malformed input rejection
- no panic on fuzz regression inputs
- decoder bounds

## Layer 7: Evidence and release

The repository records reproducibility and publication evidence.

Review focus:

- `scripts/check-production-ready.sh`
- release candidate files
- GitHub Release publication evidence
- checksums
- SRS placeholder policy
