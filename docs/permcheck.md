# Permutation checks: products and rational fingerprints

Two columns `a` and `b` are permutations when they contain the same multiset. A random challenge `β` turns this into a fingerprint comparison.

## Product fingerprint

```text
Πᵢ (β + aᵢ) = Πᵢ (β + bᵢ).
```

This is the familiar grand-product identity. In streamed implementations, constructing or reducing product layers can require intermediate buffers and repeated reads/writes.

## Rational fingerprint

Take the logarithmic derivative of the characteristic product:

```text
Σᵢ 1/(β + aᵢ) = Σᵢ 1/(β + bᵢ).
```

Each side can be accumulated in a single pass with constant live state. The trade-off is inversion work (normally batchable) and the need to avoid poles where `β = -aᵢ`.

One challenge gives a probabilistic identity test, not a standalone production argument. Real systems bind columns to commitments, derive challenges cryptographically, manage extension-field soundness, and prove the fingerprint relation inside the relevant polynomial protocol.
