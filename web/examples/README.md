# Interactive transcript fixtures

The visualizer ships with four built-in, deterministic examples:

- **Sumcheck:** evaluations `[3, 1, 4, 1, 5, 9, 2, 6]`, with every prover message and verifier challenge exposed.
- **Zerocheck:** an eight-cell constraint table whose third value can be toggled between zero and a violation.
- **PermCheck:** two permuted columns that can be mutated, checked with both product and rational fingerprints.
- **Scribe I/O:** a scalable logical traffic model from 2¹⁰ through 2²⁶ field elements.

All examples use the educational field F₉₇ and run entirely in the browser.
