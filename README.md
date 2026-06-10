<p align="center">
  <img src="docs/assets/snark-lab-demo.svg" alt="snark-lab interactive protocol demo" width="100%" />
</p>

# snark-lab: Learn and benchmark SNARK protocols from first principles

**Interactive Rust + TypeScript visualizations for Sumcheck, Zerocheck, Permutation Checks, and streaming/I/O bottlenecks in modern SNARK provers.**

Most protocol explanations stop at notation. `snark-lab` gives every symbol a state you can inspect: edit an evaluation table, reveal prover messages round by round, mutate a permutation, and scale a streaming cost model from 1K to 67M elements.

> This is an educational and research workbench, not a production cryptographic library. It uses the tiny field F₉₇ and a reproducible, non-cryptographic transcript.

## What can I learn here?

| Question | Interactive answer | Rust implementation |
|---|---|---|
| How does sumcheck work? | Watch each variable fold away and inspect every consistency check. | [`crates/sumcheck`](crates/sumcheck) |
| How does zerocheck reduce to sumcheck? | Toggle a violated constraint and see the equality-weighted claim fail. | [`crates/zerocheck`](crates/zerocheck) |
| How does permutation checking work? | Compare product and logarithmic-derivative fingerprints. | [`crates/permcheck`](crates/permcheck) |
| Why does Scribe become I/O-heavy? | Scale a product tree and see logical reads/writes dominate. | [`docs/scribe-bottleneck.md`](docs/scribe-bottleneck.md) |
| How can rational PermCheck help? | Compare repeated tree passes with one constant-state stream. | Visualizer + benchmark cost model |

## Launch the lab

```bash
git clone https://github.com/example/snark-lab.git
cd snark-lab
cargo test --workspace

cd web/visualizer
npm install
npm run dev
```

Open `http://localhost:5173`. The four labs run locally in the browser; no wallet, backend, or telemetry is involved.

## Benchmark the bottleneck

```bash
cargo run --release -p snark-lab-benches -- 20
```

The optional argument is `log₂(N)` (capped at 24). Output combines directly measured fingerprint runtimes with an explicit logical I/O model. It does **not** pretend modeled bytes are hardware counters.

## Architecture

```text
snark-lab/
├── crates/
│   ├── field/          # F_97 arithmetic
│   ├── multilinear/    # dense MLEs and eq polynomial
│   ├── sumcheck/       # prover, verifier, inspectable transcript
│   ├── zerocheck/      # equality-weighted reduction
│   ├── permcheck/      # product/rational fingerprints + cost model
│   └── benches/        # runtime and logical memory/I/O comparison
├── web/
│   ├── visualizer/     # responsive React + TypeScript workbench
│   └── examples/       # deterministic transcript fixtures
├── notebooks/          # worked protocol experiments
└── docs/               # protocol and bottleneck notes
```

## Protocol map

```text
constraint table ──eq(τ,x)──▶ weighted polynomial ──▶ SUMCHECK
                                                        │
witness columns ──β──▶ product fingerprint ─────────────┤
              └──β──▶ rational fingerprint              │
                                                        ▼
                                              one final oracle check
```

## Design principles

- **Inspectable over magical.** Proof rounds are plain structs, not opaque byte strings.
- **Honest about soundness.** Docs identify where commitments, Fiat–Shamir, larger fields, and batching belong.
- **Measured vs. modeled.** Benchmark output labels runtime measurements separately from logical I/O estimates.
- **One concept per crate.** The dependency graph mirrors the protocol reductions.

## Documentation

- [Sumcheck from first principles](docs/sumcheck.md)
- [Zerocheck as weighted sumcheck](docs/zerocheck.md)
- [Product and rational PermCheck](docs/permcheck.md)
- [The Scribe streaming bottleneck](docs/scribe-bottleneck.md)
- [Worked examples notebook](notebooks/protocol-examples.md)

## What’s next?

The current lab establishes the protocol core. The highest-value follow-up milestones are:

1. **Commitment-backed Sumcheck:** replace direct table access with polynomial commitment openings and a cryptographic Fiat–Shamir transcript.
2. **Sounder PermCheck experiments:** add extension-field challenges, batched inversion, denominator constraints, and repeated-challenge error analysis.
3. **Real systems measurements:** supplement the transparent I/O model with RSS, cache-miss, disk-throughput, and flamegraph capture on reproducible workloads.
4. **Transcript interchange:** export every browser experiment as JSON and replay it through the Rust verifier.
5. **Larger protocol compositions:** visualize GKR-style layered reductions and connect Zerocheck/PermCheck to a small end-to-end proving pipeline.

See the issue tracker for scoped tasks; contributions should preserve the distinction between educational models and production-safe cryptography.

## Contributing

Interesting next steps include extension fields, batched inversion, commitment-backed oracle openings, real memory counters, transcript export/import, and adapters for production proof systems. Keep examples deterministic and explain every optimization's effect on soundness.

Licensed under MIT or Apache-2.0.
