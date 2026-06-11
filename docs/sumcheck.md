# Fiat–Shamir Sumcheck over large prime fields

The Rust `sumcheck` crate is generic over `ark_ff::PrimeField`; tests and benchmarks instantiate it with the BLS12-381 scalar field. For a multilinear table `f : {0,1}ⁿ → F`, the prover claims

```text
S = Σ_{x∈{0,1}ⁿ} f(x).
```

At round `i`, the prover sends the degree-one polynomial `gᵢ`. The verifier checks

```text
gᵢ(0) + gᵢ(1) = current_claim.
```

The implementation then appends both evaluations of `gᵢ` to a domain-separated Merlin transcript and derives `rᵢ`. Only after that does the protocol update the claim to `gᵢ(rᵢ)`. The proof stores round messages, not prover-selected challenges.

The transcript binds the protocol domain, variable count, claimed sum, transparent oracle table, round index, and each round polynomial. Tests establish that the same transcript reproduces challenges and that changing public input or an earlier round message changes subsequent challenges.

## Current oracle boundary

This crate currently receives the full multilinear table. That makes the algebra and Fiat–Shamir ordering real, but verification is not succinct: the verifier directly evaluates the table at `(r₁,…,rₙ)`. A production proof system must replace transparent table binding and direct evaluation with a commitment and an authenticated opening.

```bash
cargo test -p sumcheck
cargo run --release -p snark-lab-benches -- 18
```
