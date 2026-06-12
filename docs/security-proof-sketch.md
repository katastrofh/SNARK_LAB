# Security Proof Sketch

This document gives a proof-outline for the security properties targeted by SNARK_LAB.

It is not a formal machine-checked proof. It is a structured argument documenting the assumptions, reductions, and implementation boundaries used by the repository.

## Classification

SNARK_LAB is currently:

- a production-grade research prototype
- serious protocol engineering infrastructure
- not audited
- not production-secure deployment software

No part of this document should be read as an audit result or as deployment approval.

## Global assumptions

The proof sketches below assume:

- the selected finite field has sufficient size for the target soundness level
- Arkworks field and curve operations are correct
- Merlin transcript binding is collision-resistant in the Fiat-Shamir model
- SHA-256 is collision-resistant where used for SRS digests
- canonical serialization is injective for accepted values
- invalid encodings are rejected
- unsafe Rust is not introduced into repository crates
- production SRS material is generated outside this repository by a sound ceremony or derivation process
- known-discrete-log test fixture material is never accepted as production SRS material

## Fiat-Shamir transcript ordering

### Claim

If every challenge is sampled only after the relevant statement and previous prover messages have been transcript-bound, then a prover cannot adapt earlier messages after seeing later challenges without changing the transcript.

### Argument

Each protocol phase appends public statement data and prover messages before deriving the next challenge.

If an adversary changes a prior statement or message, the transcript state changes. Under the Fiat-Shamir random-oracle heuristic, the resulting challenge is computationally unpredictable and independent from the previous challenge, except with negligible collision probability.

Therefore, transcript order enforces non-adaptivity of earlier messages relative to later challenges.

### Implementation boundary

This repository tests that changed statements and changed prover messages alter downstream challenges.

The property depends on all protocol entry points using the intended transcript binding functions.

## Sumcheck

### Claim

For a multilinear polynomial table over a finite field, the implemented Sumcheck verifier rejects false claims except with probability bounded by the standard Sumcheck soundness error under Fiat-Shamir challenge sampling.

### Argument

At each round, the prover sends a univariate round polynomial. The verifier checks consistency of the previous claim with the two Boolean evaluations of the current round polynomial. The verifier then samples a Fiat-Shamir challenge and reduces the claim dimension by one.

If the prover's claimed sum is false, then at some round the prover must send a polynomial that disagrees with the true restricted polynomial. The verifier samples a challenge after the message is fixed. By Schwartz-Zippel, the probability that a nonzero disagreement polynomial vanishes at the sampled challenge is bounded by its degree divided by the field size.

Repeating across rounds gives the standard Sumcheck error bound.

### Implementation boundary

The implementation checks:

- claimed sum consistency
- round polynomial degree/shape
- transcript-bound challenge derivation
- final evaluation consistency
- malformed proof rejection

## Zerocheck

### Claim

If the constraint table is zero on the Boolean hypercube, Zerocheck accepts. If the table is nonzero, the reduction to Sumcheck rejects except with the Sumcheck soundness error plus the equality-mixing error.

### Argument

Zerocheck binds the constraint oracle before deriving the random equality-mixing point. The reduction constructs a weighted Sumcheck statement where the zero constraint condition is transformed into a sumcheckable identity.

If the constraint table is not identically zero, then with high probability over the mixing point the weighted reduction is nonzero. The resulting false Sumcheck claim is rejected by the Sumcheck argument.

### Implementation boundary

The implementation ensures:

- constraint oracle is bound before the mixing point
- zero tables verify
- nonzero tables are rejected
- challenge derivation is statement-bound

## PermCheck product fingerprint

### Claim

If two tagged multisets are equal, product fingerprints match. If they differ, product fingerprints match only with probability bounded by the degree of the induced polynomial identity divided by the field size.

### Argument

The product fingerprint compresses tagged values using random transcript challenges beta and gamma. Equal multisets produce identical products by commutativity.

For unequal multisets, equality of products defines a nonzero polynomial equation in the sampled challenges, except in degenerate cases. Since beta and gamma are sampled after the tagged columns are bound, the adversary cannot adapt the columns to the sampled challenges. By Schwartz-Zippel, false equality occurs with probability at most degree over field size.

### Implementation boundary

The implementation binds tagged columns before deriving beta and gamma.

## PermCheck rational fingerprint

### Claim

The rational fingerprint is equivalent to checking equality of tagged denominator multisets, except when a denominator pole occurs. Poles are rejected explicitly.

### Argument

The rational check compares sums of inverses of randomized tagged terms. For matching multisets, inverse sums agree. For distinct multisets and nonzero denominators, equality again defines a nonzero rational identity. After clearing denominators, the false-accept condition is a nonzero polynomial identity, bounded by Schwartz-Zippel.

The implementation rejects denominator poles instead of silently treating them as valid arithmetic.

### Implementation boundary

The implementation exposes denominator-pole errors and tests pole rejection.

## IPA commitment equation

### Claim

Given a valid generator basis and blinding generator, the IPA commitment binds a polynomial vector and blinding scalar through the relation:

C = inner_product(a, G) + rH

where:

- C is the curve commitment
- a is the polynomial evaluation vector
- G is the polynomial generator vector
- r is the blinding scalar
- H is the blinding generator

### Argument

The verifier can check algebraic consistency where witness material is available, and later opening proofs show that the committed vector evaluates correctly at a requested point.

Binding depends on the discrete-log hardness of the curve group and independence of the generator basis. If generator relations are known, binding may fail. Therefore, production SRS provenance is required.

### Implementation boundary

The implementation checks:

- generator count
- point validity
- identity rejection
- duplicate rejection
- commitment equation consistency
- canonical commitment serialization

## IPA opening relation

### Claim

For a committed vector a and evaluation basis b derived from an opening point z, the IPA opening proves that:

v = inner_product(a, b)

and that a is the vector committed in C.

### Argument

The prover and verifier reduce the vector relation by repeated inner-product folding. At each round, the prover sends L and R commitments. The verifier absorbs them into the transcript, samples challenge x, and folds the relation.

The recursive invariant is that the folded commitment relation preserves the same claimed inner-product relation under the challenge. After all rounds, the relation reduces to one scalar multiplication equation involving the final folded generators, final scalar witnesses, and the inner-product generator.

If a prover tampers with a round commitment, final scalar, public commitment, claimed value, opening point, or transcript label, the recursive relation fails except with the Fiat-Shamir soundness error.

### Implementation boundary

The implementation includes:

- L/R round commitments
- transcript-bound round challenges
- polynomial vector folding
- evaluation vector folding
- generator folding
- final commitment relation check
- proof shape validation
- negative malformed-proof fixtures
- randomized roundtrip tests

## Blinded IPA opening

### Claim

The blinded opening path preserves the same evaluation value while hiding the original committed vector through an explicit extension.

### Argument

The blinded relation extends the polynomial vector with the blinding scalar and extends the generator basis so that the original commitment becomes an unblinded commitment in the extended basis.

The evaluation basis is extended with zero in the blinding coordinate, so the claimed evaluation value is unchanged.

This allows the standard IPA opening verifier to check the extended relation without learning the blinding scalar.

### Implementation boundary

The implementation checks:

- extended generator counts
- padding generator counts
- public commitment consistency
- wrong padding rejection
- wrong public commitment rejection
- tampered final scalar rejection

## IPA serialization

### Claim

Accepted serialized proofs and openings decode into a unique typed representation, and malformed encodings fail closed.

### Argument

Each encoded object includes magic bytes, explicit lengths, canonical field encodings, and trailing-byte rejection. Any truncated, extended, or wrong-magic encoding is rejected before verification.

A successful decode is not proof acceptance. Verification remains a separate cryptographic check.

### Implementation boundary

The implementation tests:

- wrong magic rejection
- truncation rejection
- trailing-byte rejection
- corrupt inner proof rejection
- claimed-value mismatch rejection
- fuzz target compilation for parser hardening

## IPA SRS provenance

### Claim

A loaded IPA SRS is accepted only if the generator material matches its provenance metadata and canonical digest.

### Argument

The SRS loader decodes canonical curve points, rejects invalid points, rejects identities, rejects duplicates, checks generator counts, and computes a canonical digest over the accepted basis. The digest must match the provenance record.

Known-discrete-log test fixture provenance is rejected for production validation.

### Implementation boundary

The implementation validates:

- external trusted setup provenance
- hash-to-curve derivation provenance
- nonzero digests
- matching variable count
- matching canonical digest
- malformed SRS file rejection
- CLI SRS validation

## Transparent oracle boundary

Some components still expose transparent-oracle paths for research and educational purposes.

These paths bind full tables into the transcript and may allow direct verifier evaluation. They are useful for protocol development and testing, but they are not a succinct production SNARK backend.

## Browser visualizer boundary

The browser visualizer is educational. It may use small readable fields and deterministic UI fixtures.

Browser arithmetic and educational interchange files are not the same as the Rust cryptographic transcript path.

## Remaining gaps

The following are still required before stronger security claims:

- external cryptographic audit
- public security theorem document
- independent reference implementation comparison
- long-running fuzz campaign artifacts
- public regression corpus
- production SRS ceremony or derivation process
- side-channel review
- constant-time review
- dependency trust review
- release reproducibility process

## Summary

The repository now has a coherent proof story for the implemented research prototype:

- statements and messages are transcript-bound before challenges
- Sumcheck follows the standard round reduction argument
- Zerocheck reduces to transcript-ordered Sumcheck
- PermCheck uses randomized product/rational fingerprints
- IPA proves committed vector openings through recursive folding
- SRS material is accepted only with explicit provenance and matching digest
- malformed external bytes fail closed

The project remains unaudited and should not be used as production deployment software.
