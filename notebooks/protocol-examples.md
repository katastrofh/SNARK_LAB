# Protocol examples notebook

All arithmetic below is modulo 97.

## Sumcheck transcript

Let the two-variable multilinear table be `[1, 2, 3, 4]`. Its Boolean sum is `10`.

- Round 1: `g₁(0)=1+3=4`, `g₁(1)=2+4=6`; check `4+6=10`.
- The verifier samples `r₁` and updates to `g₁(r₁)=4(1-r₁)+6r₁`.
- Fold pairs `[1,2]` and `[3,4]` at `r₁`.
- Round 2 repeats the same check on the two folded values.
- The final scalar equals the multilinear evaluation at `(r₁,r₂)`.

Reproduce the exact deterministic transcript with:

```rust
let p = Multilinear::new(vec![1.into(), 2.into(), 3.into(), 4.into()])?;
let proof = sumcheck::prove(&p, p.sum_hypercube());
sumcheck::verify(&p, &proof)?;
```

## Zerocheck experiment

Start from `[0,0,0,0]`, choose a mixing point, and verify the zero claim. Change one cell to `9`; the weighted sumcheck proof generated against claim zero fails its first consistency check.

## PermCheck experiment

Compare `a=[1,5,9,2]` and `b=[9,2,1,5]` at a non-pole challenge. Both product and rational fingerprints match. Replace the last `5` with `6`; both usually differ.

## I/O experiment

```bash
cargo run --release -p snark-lab-benches -- 20
```

The runtime values measure this small implementation. The reported bytes are the documented logical traffic model, making assumptions explicit and reproducible.
