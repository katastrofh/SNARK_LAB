# Interactive transcript fixtures

The visualizer ships with four built-in, deterministic examples:

- **Sumcheck:** evaluations `[3, 1, 4, 1, 5, 9, 2, 6]`, with every prover message and verifier challenge exposed.
- **Zerocheck:** an eight-cell constraint table whose third value can be toggled between zero and a violation.
- **PermCheck:** two permuted columns that can be mutated, checked with both product and rational fingerprints.
- **Scribe I/O:** a scalable logical traffic model from 2¹⁰ through 2²⁶ field elements.

All browser examples use the educational field F₉₇ and run entirely in the browser. The Rust protocol crates separately use Arkworks large prime fields and Merlin Fiat–Shamir transcripts.

## Machine-checkable fixtures

Versioned JSON transcripts live in [`../../examples/transcripts`](../../examples/transcripts). The browser's **Export transcript JSON** action emits the same schema, and `snark-lab-cli verify-transcript` verifies it against the Rust implementation. Both valid and deliberately tampered fixtures are covered by workspace tests.
