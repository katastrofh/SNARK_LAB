# Sumcheck from first principles

Sumcheck lets a prover convince a verifier that a low-degree polynomial `f` has a claimed sum over the Boolean hypercube:

```text
H = Σ_{x ∈ {0,1}ⁿ} f(x).
```

The lab uses a multilinear polynomial represented by its `2ⁿ` evaluations. In round `i`, the prover sends the univariate polynomial obtained by summing over every variable except `xᵢ`. Because the table is multilinear, two values describe that line: `gᵢ(0)` and `gᵢ(1)`.

The verifier checks `gᵢ(0) + gᵢ(1) = Hᵢ₋₁`, samples challenge `rᵢ`, and updates the claim to `Hᵢ = gᵢ(rᵢ)`. After `n` rounds, the verifier checks the final claim against one evaluation `f(r₁,…,rₙ)`.

## Run it

```bash
cargo test -p sumcheck
cargo run -p snark-lab-benches -- 18
cd web/visualizer && npm run dev
```

## Educational boundary

The transcript challenge in this repository is deterministic so examples reproduce exactly. A production non-interactive proof must use a cryptographic transcript/Fiat–Shamir construction and a commitment opening for the final oracle evaluation.
