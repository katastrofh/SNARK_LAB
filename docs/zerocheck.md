# Zerocheck as transcript-ordered weighted Sumcheck

Zerocheck proves that `f(x)=0` for every Boolean point. It reduces this to

```text
Σ_x eq(τ,x) · f(x) = 0,
eq(τ,x) = Πᵢ [τᵢxᵢ + (1-τᵢ)(1-xᵢ)].
```

The security-critical order in the Rust crate is:

1. domain-separate Zerocheck;
2. bind the constraint oracle and its dimensions;
3. derive the mixing coordinates `τ` from Merlin;
4. construct the equality-weighted multilinear table;
5. invoke Fiat–Shamir Sumcheck on the zero claim.

Thus a prover cannot choose the constraint table after learning `τ`. The current binding is transparent because the full table is appended. A commitment-backed version will append an oracle commitment at step 2 and verify openings after Sumcheck.

Tests use BLS12-381 `Fr`, accept an all-zero table, reject a nonzero constraint table, and confirm that changing the bound constraint oracle changes `τ`.
