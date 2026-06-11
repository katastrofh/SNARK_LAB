# Fiat-Shamir Sumcheck over large prime fields

The Rust `sumcheck` crate is generic over `ark_ff::PrimeField`; tests and benchmarks instantiate it with the BLS12-381 scalar field.

For a multilinear table `f : {0,1}^n -> F`, the prover claims:

    S = sum_{x in {0,1}^n} f(x).

At round `i`, the prover sends the degree-one polynomial `g_i`. The verifier checks:

    g_i(0) + g_i(1) = current_claim.

Only after binding this prover message does the verifier derive the next challenge `r_i`. The claim is then reduced to:

    current_claim <- g_i(r_i).

After `n` rounds, the verifier checks the final folded claim against the oracle evaluation at `(r_1,...,r_n)`.

## Rust path

The Rust implementation binds the following into the transcript:

- protocol domain,
- field modulus,
- number of variables,
- claimed sum,
- transparent oracle table,
- round index,
- `g_i(0)`,
- `g_i(1)`.

Then it derives each challenge through the transcript. The proof stores prover messages, not prover-selected randomness.

## Browser path

The browser visualizer is an inspectable `F_97` model. It uses deterministic challenges so that examples are reproducible, exportable, and easy to inspect.

The browser is not a production verifier. It is a protocol workbench.

## Attack modes to add next

| Mode | What it changes | Expected failure |
|---|---|---|
| Honest proof | Nothing | Accepts |
| Wrong claimed sum | Starts with an off-by-one claim | First endpoint check fails |
| Tamper round polynomial | Changes `g_1(1)` | Round consistency fails |
| Tamper final opening | Changes the final oracle value | Final oracle check fails |
| Mutate oracle table | Checks the final point against a changed table | Final oracle check fails |

These modes demonstrate why the order matters:

    bind message -> check endpoints -> sample challenge -> fold claim

A dishonest prover must commit before seeing the fresh challenge.

## Current oracle boundary

This crate currently supports a transparent oracle path: the verifier can directly evaluate the table at `(r_1,...,r_n)`. That makes the algebra and Fiat-Shamir ordering real, but verification is not succinct.

A production proof system must replace transparent table binding and direct table evaluation with:

    oracle commitment + authenticated opening proof

The `snark-lab-oracle` crate is the abstraction boundary for this next step.

## Run

    cargo test -p sumcheck
    cargo run --release -p snark-lab-benches -- 18
