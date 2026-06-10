# Zerocheck as weighted sumcheck

Zerocheck proves that a constraint polynomial vanishes at every point of the Boolean hypercube:

```text
f(x) = 0 for all x ∈ {0,1}ⁿ.
```

Checking all `2ⁿ` values defeats succinct verification. Instead, sample a random mixing point `τ` and use the multilinear equality polynomial

```text
eq(τ, x) = Πᵢ [τᵢxᵢ + (1-τᵢ)(1-xᵢ)].
```

The prover and verifier invoke sumcheck on

```text
Σ_x eq(τ,x) f(x) = 0.
```

If every constraint value is zero, the claim is always true. If any value is nonzero, the random weighting makes cancellation unlikely. The `zerocheck` crate constructs the weighted evaluation table and delegates its transcript to the generic `sumcheck` crate.

The web lab lets you toggle one bad constraint and watch the ordinary sumcheck reduction reject the zero claim.
